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
