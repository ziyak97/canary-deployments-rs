# GitHub Release Automation

This is a Rust program that automates the creation of releases on GitHub. It uses the `octocrab` crate to interact with the GitHub API.

## Usage

To use this program, you will need to set the `GITHUB_REPOSITORY` and `GITHUB_TOKEN` environment variables. The `GITHUB_REPOSITORY` variable should be set to the repository for which you want to create releases (e.g. `owner/repo`). The `GITHUB_TOKEN` variable should be set to a personal access token with the appropriate permissions.

You can run the program using the following command:

`cargo run -- [release_type] [semantic_version_type]`

The `[release_type]` argument should be either `canary` or `release`. The `[semantic_version_type]` argument should be one of `major`, `minor`, or `patch`.

If `[release_type]` is set to `canary`, the program will create a new canary release. If `[release_type]` is set to `release`, the program will create a new stable release.

The program will automatically generate release notes based on merged pull requests and group them by label. It will also generate a list of contributors.

## Example

To create a new canary release with a minor version bump, you would run the following command:

`cargo run -- canary minor`

To create a new stable release, you would run the following command:

`cargo run -- release`

## How it works

The script reads information about the latest release and merged pull requests from a specified GitHub repository. Based on this information, it generates release notes and a list of contributors and creates a new canary or stable release.

The script uses semantic versioning to determine the version number of the new release. When creating a canary release, you can specify whether the release should increment the major, minor, or patch version number. For example:

- If the latest release is `v1.2.3` and you create a new canary release with the `major` semantic version type, the new release will have the version number `v2.0.0-canary.0`.
- If the latest release is `v1.2.3` and you create a new canary release with the `minor` semantic version type, the new release will have the version number `v1.3.0-canary.0`.
- If the latest release is `v1.2.3` and you create a new canary release with the `patch` semantic version type, the new release will have the version number `v1.2.4-canary.0`.

When creating a stable release, the script will use the version number of the latest canary release without the `-canary.X` suffix.

The first release created by the script must be a canary release. The initial canary release will be based on the specified semantic version type (`major`, `minor`, or `patch`). For example:

- If there are no existing releases and you create a new canary release with the `major` semantic version type, the new release will have the version number `v1.0.0-canary.0`.
- If there are no existing releases and you create a new canary release with the `minor` semantic version type, the new release will have the version number `v0.1.0-canary.0`.
- If there are no existing releases and you create a new canary release with the `patch` semantic version type, the new release will have the version number `v0.0.1-canary.0`.

The script groups merged pull requests into different sections of the release notes based on their labels. To include a pull request in a specific section of the release notes, add one of the following labels to it:

- `area:core`: Pull requests with this label will be included in the "Core Changes" section of the release notes.
- `area:documentation`: Pull requests with this label will be included in the "Documentation Changes" section of the release notes.
- Other: Pull requests without either of these labels will be included in the "Miscellaneous Changes" section of the release notes.

## How to use this program in your own project

1. Create a personal access token on GitHub with the `repo` or `public_repo` scope. To do this, go to your GitHub account settings, click on "Developer settings" in the left sidebar, click on "Personal access tokens", and then click on the "Generate new token" button. Enter a note to describe the purpose of the token and select the appropriate scope:

   - If your repository is **private**, select the `repo` scope to give the token full access to your private repositories.
   - If your repository is **public**, select the `public_repo` scope to give the token access to only public repositories.

   Click on the "Generate token" button and copy the generated token to your clipboard.

2. Save the personal access token as a secret in your repository. To do this, go to the main page of your repository on GitHub, click on the "Settings" tab, click on "Secrets" in the left sidebar, and then click on the "New repository secret" button. Enter `PERSONAL_ACCESS_TOKEN` as the name of the secret and paste the personal access token into the value field. Click on the "Add secret" button to save the secret.

3. Create a new file in your repository's `.github/workflows` directory and name it `release.yml`. Paste the following content into the file:

```yaml
name: Release
on:
  workflow_dispatch:
    inputs:
      releaseType:
        description: "Release type (canary or release)"
        required: true
        type: choice
        options:
          - canary
          - release
      semanticVersionType:
        description: "Semantic version type (major, minor, or patch)"
        type: choice
        options:
          - patch
          - minor
          - major
jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - name: Download binary
        run: |
          curl -L -o canary-deployments-rs https://github.com/ziyak97/canary-deployments-rs/releases/download/v0.0.1/canary-deployments-rs
          chmod +x canary-deployments-rs
      - name: Run script
        run: ./canary-deployments-rs ${{ github.event.inputs.releaseType }} ${{ github.event.inputs.semanticVersionType }}
        env:
          GITHUB_TOKEN: ${{ secrets.PERSONAL_ACCESS_TOKEN }}
```

4. Commit and push your changes to save the workflow file.

After completing these steps, you will be able to manually trigger the workflow from the “Actions” tab of your repository on GitHub. Select the “Release” workflow from the list of workflows and click on the “Run workflow” button. Choose the release type and semantic version type from the dropdown menus and click on the “Run workflow” button to start the workflow.

## Building the program

To build the program, run the following command:

```cargo build --release --target x86_64-unknown-linux-gnu```

The compiled binary will be located at `target/x86_64-unknown-linux-gnu/release/canary-deployments-rs`.

This will build it to support Linux. If you want to build it to support a different platform, replace `x86_64-unknown-linux-gnu` with the appropriate target triple. You can find a list of supported target triples [here](https://forge.rust-lang.org/release/platform-support.html).