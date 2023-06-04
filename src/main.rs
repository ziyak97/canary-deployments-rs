use octocrab::models::issues::Issue;
use octocrab::{models, Octocrab};
use std::collections::HashSet;
use std::env;
use regex::Regex;
use tokio;

async fn get_last_stable_release(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
) -> octocrab::Result<Option<models::repos::Release>> {
    let mut page = octocrab
        .repos(owner, repo)
        .releases()
        .list()
        .per_page(100)
        .send()
        .await?;

    let mut releases = page.take_items().into_iter();

    while let Some(release) = releases.next() {
        if !release.prerelease {
            return Ok(Some(release));
        }
    }

    Ok(None)
}

async fn get_latest_release(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
) -> octocrab::Result<Option<models::repos::Release>> {
    let release = octocrab.repos(owner, repo).releases().get_latest().await;

    match release {
        Ok(release) => Ok(Some(release)),
        Err(_) => Ok(None),
    }
}

async fn get_merged_pull_requests(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    published_at: &str,
) -> octocrab::Result<Vec<models::issues::Issue>> {
    let query = format!(
        "repo:{}/{} is:pr is:merged merged:>{}",
        owner, repo, published_at
    );
    let response = octocrab
        .search()
        .issues_and_pull_requests(&query)
        .send()
        .await?;

    // filter out issues and return only pull requests
    let response = response
        .items
        .into_iter()
        .filter(|issue| issue.pull_request.is_some())
        .collect::<Vec<Issue>>();

    Ok(response)
}

fn group_pull_requests_by_label(pull_requests: &Vec<Issue>) -> (Vec<&Issue>, Vec<&Issue>, Vec<&Issue>) {
    let mut core_changes = Vec::new();
    let mut documentation_changes = Vec::new();
    let mut miscellaneous_changes = Vec::new();

    for pr in pull_requests {
        if pr.labels.iter().any(|label| label.name == "area:core") {
            core_changes.push(pr);
        } else if pr
            .labels
            .iter()
            .any(|label| label.name == "area:documentation")
        {
            documentation_changes.push(pr);
        } else {
            miscellaneous_changes.push(pr);
        }
    }

    (core_changes, documentation_changes, miscellaneous_changes)
}

fn generate_release_notes(
    core_changes: Vec<&Issue>,
    documentation_changes: Vec<&Issue>,
    miscellaneous_changes: Vec<&Issue>,
) -> String {
    let mut release_notes = String::new();

    if !core_changes.is_empty() {
        release_notes.push_str("## Core Changes\n");

        for pr in core_changes {
            release_notes.push_str(&format!("- {}: #{}\n", pr.title, pr.number));
        }

        release_notes.push('\n');
    }

    if !documentation_changes.is_empty() {
        release_notes.push_str("## Documentation Changes\n");

        for pr in documentation_changes {
            release_notes.push_str(&format!("- {}: #{}\n", pr.title, pr.number));
        }

        release_notes.push('\n');
    }

    if !miscellaneous_changes.is_empty() {
        release_notes.push_str("## Miscellaneous Changes\n");

        for pr in miscellaneous_changes {
            release_notes.push_str(&format!("- {}: #{}\n", pr.title, pr.number));
        }

        release_notes.push('\n');
    }

    release_notes
}

fn generate_contributors_list(merged_pull_requests: Vec<Issue>) -> String {
    let mut contributors = HashSet::new();

    for pr in merged_pull_requests {
        contributors.insert(pr.user.login);
    }

    let mut contributors_list = String::new();

    if !contributors.is_empty() {
        contributors_list.push_str("## Contributors\n");
        contributors_list.push_str("A big thank you to our ");
        let contributors_vec: Vec<String> = contributors.into_iter().collect();

        if contributors_vec.len() == 1 {
            contributors_list.push_str(&format!("contributor @{}.", contributors_vec[0]));
        } else {
            contributors_list.push_str("contributors ");

            for (index, contributor) in contributors_vec.iter().enumerate() {
                if index == contributors_vec.len() - 1 {
                    contributors_list.push_str(&format!("and @{}.", contributor));
                } else {
                    contributors_list.push_str(&format!("@{}, ", contributor));
                }
            }
        }

        contributors_list.push('\n');
    }

    contributors_list
}

async fn create_canary_release(octocrab: &Octocrab) -> octocrab::Result<()> {
    let repository = env::var("GITHUB_REPOSITORY").unwrap();
    let semantic_version_type = env::args().nth(2).unwrap();

    let (owner, repo) = {
        let mut split = repository.split('/');
        (split.next().unwrap(), split.next().unwrap())
    };

    let latest_release = get_latest_release(octocrab, owner, repo).await?;

    let mut tag_name = "v0.0.0-canary.0".to_string();
    let mut release_notes = String::new();
    if let Some(latest_release) = &latest_release {
        if let Some(captures) = Regex::new(r"(v\d+)\.(\d+)\.(\d+)")
            .unwrap()
            .captures(&latest_release.tag_name)
        {
            let major = captures[1][1..].parse::<u32>().unwrap();
            let minor = captures[2].parse::<u32>().unwrap();
            let patch = captures[3].parse::<u32>().unwrap();
            let is_canary = latest_release.prerelease;
            let canary_version = if is_canary {
                latest_release
                    .tag_name
                    .split("canary.")
                    .last()
                    .and_then(|s| s.parse::<u32>().ok())
            } else {
                None
            };

            if is_canary {
                if let Some(canary_version) = canary_version {
                    tag_name = format!(
                        "v{}.{}.{}-canary.{}",
                        major,
                        minor,
                        patch,
                        canary_version + 1
                    );
                }
            } else {
                // Increment version number based on semantic version type
                match semantic_version_type.as_str() {
                    "major" => tag_name = format!("v{}.0.0-canary.0", major + 1),
                    "minor" => tag_name = format!("v{}.{}.0-canary.0", major, minor + 1),
                    "patch" => tag_name = format!("v{}.{}.{}-canary.0", major, minor, patch + 1),
                    _ => {}
                }
            }
        }

        let published_at_string = latest_release
            .published_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| "0".to_string());
            
        let published_at = published_at_string.as_str();

        // Get merged pull requests between latest release and new canary release
        let merged_pull_requests =
            get_merged_pull_requests(octocrab, owner, repo, published_at).await?;

        // Guard clause: No merged pull requests
        if merged_pull_requests.is_empty() {
            println!("No merged pull requests found between latest release and new canary release");
            return Ok(());
        }

        // Group pull requests by label
        let (core_changes, documentation_changes, miscellaneous_changes) =
            group_pull_requests_by_label(&merged_pull_requests);

        // Generate release notes
        release_notes.push_str(&generate_release_notes(
            core_changes,
            documentation_changes,
            miscellaneous_changes,
        ));

        // Generate list of contributors
        release_notes.push_str(&generate_contributors_list(merged_pull_requests));
    } else {
        // No releases found for repository
        match semantic_version_type.as_str() {
            "major" => tag_name = "v1.0.0-canary.0".to_string(),
            "minor" => tag_name = "v0.1.0-canary.0".to_string(),
            "patch" => tag_name = "v0.0.1-canary.0".to_string(),
            _ => {}
        }
    }

    let name = tag_name.clone();
    let body = format!(
        "New canary release based on {}\n\n{}",
        latest_release.map_or_else(|| "".to_string(), |r| r.tag_name),
        release_notes
    );
    octocrab
        .repos(owner, repo)
        .releases()
        .create(tag_name.as_str())
        .name(name.as_str())
        .body(body.as_str())
        .prerelease(true)
        .send()
        .await?;

    Ok(())
}

async fn create_release(octocrab: &Octocrab) -> octocrab::Result<()> {
    let repository = env::var("GITHUB_REPOSITORY").unwrap();
    let (owner, repo) = {
        let mut split = repository.split('/');
        (split.next().unwrap(), split.next().unwrap())
    };

    let latest_release = get_latest_release(octocrab, owner, repo).await?;
    let latest_canary_release = latest_release.filter(|r| r.prerelease);

    if latest_canary_release.is_none() {
        println!("No canary releases found for repository");
        return Ok(());
    }

    let latest_canary_release = latest_canary_release.unwrap();

    let tag_name = latest_canary_release
        .tag_name
        .split("-canary")
        .next()
        .unwrap();

    let name = tag_name;

    let last_stable_release = get_last_stable_release(octocrab, owner, repo).await?;

    if last_stable_release.is_none() {
        println!("No stable releases found for repository");
        return Ok(());
    }

    let last_stable_release = last_stable_release.unwrap();

    let published_at_string = last_stable_release
        .published_at
        .map(|d| d.to_rfc3339())
        .unwrap_or_else(|| "0".to_string());

    let published_at = published_at_string.as_str();

    let merged_pull_requests =
        get_merged_pull_requests(octocrab, owner, repo, published_at).await?;

    // Guard clause: No merged pull requests
    if merged_pull_requests.is_empty() {
        println!("No merged pull requests found between latest release and new canary release");
        return Ok(());
    }

    // Group pull requests by label
    let (core_changes, documentation_changes, miscellaneous_changes) =
        group_pull_requests_by_label(&merged_pull_requests);

    // Generate release notes
    let mut release_notes =
        generate_release_notes(core_changes, documentation_changes, miscellaneous_changes);

    // Generate list of contributors
    release_notes.push_str(&generate_contributors_list(merged_pull_requests));

    octocrab
        .repos(owner, repo)
        .releases()
        .create(tag_name)
        .name(name)
        .body(release_notes.as_str())
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let release_type = env::args().nth(1).unwrap();
    let _semantic_version_type = env::args().nth(2).unwrap();

    let personal_token = env::var("GITHUB_TOKEN").unwrap();

    let octocrab = Octocrab::builder()
        .personal_token(personal_token)
        .build()?;

    if release_type == "canary" {
        create_canary_release(&octocrab).await?;
    } else if release_type == "release" {
        create_release(&octocrab).await?;
    }

    Ok(())
}
