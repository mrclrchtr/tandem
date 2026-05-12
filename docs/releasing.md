# Releasing

Releases are automated via GitHub Actions.

## Flow

1. Merge Conventional Commit PRs into `main`.
2. `release-please` opens/updates a release PR with version + changelog updates.
3. Merge the release PR.
4. `release-please` creates a `vX.Y.Z` tag.
5. `cargo-dist` builds release binaries and publishes assets to GitHub Releases.

## Version source of truth

`Cargo.toml` (`workspace.package.version`).

## GitHub App setup

`release-please` needs a GitHub App token to push tags reliably:

- Create a GitHub App with **repo contents** + **PR write** permissions.
- Configure repository variable: `RELEASE_APP_CLIENT_ID`
- Configure repository secret: `RELEASE_APP_PRIVATE_KEY`

`actions/create-github-app-token` uses these to generate a short-lived token.

## Homebrew tap publishing

Requires:

- A `mrclrchtr/homebrew-tap` repository on GitHub.
- The same GitHub App (used for release-please) installed on the tap repo with **Contents** read/write permission.
