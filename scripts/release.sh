#!/usr/bin/env bash
# ==============================================================================
# mdview - Automated Release Helper
# Usage: ./scripts/release.sh [version]
# Example: ./scripts/release.sh 1.2.0
# ==============================================================================

set -euo pipefail

RED="\033[1;31m"
GREEN="\033[1;32m"
YELLOW="\033[1;33m"
BLUE="\033[1;34m"
RESET="\033[0m"

log_info() { echo -e "${BLUE}==>${RESET} $1"; }
log_success() { echo -e "${GREEN}==>${RESET} $1"; }
log_warn() { echo -e "${YELLOW}==>${RESET} $1"; }
log_error() { echo -e "${RED}Error:${RESET} $1" >&2; exit 1; }

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Ensure git repo is clean
if [ -n "$(git status --porcelain)" ]; then
    log_error "Working directory has uncommitted changes. Please commit or stash them first."
fi

# Determine version
VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -n1 | cut -d '"' -f2)
    echo -e "Current version in Cargo.toml: ${YELLOW}${CURRENT_VERSION}${RESET}"
    read -rp "Enter release version (e.g. 1.1.1 or press enter for ${CURRENT_VERSION}): " INPUT_VERSION
    VERSION="${INPUT_VERSION:-$CURRENT_VERSION}"
fi

# Strip optional leading 'v'
VERSION="${VERSION#v}"
TAG="v${VERSION}"

# Validate SemVer format
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
    log_error "Invalid semantic version format: '${VERSION}'. Expected X.Y.Z (e.g., 1.2.0)"
fi

# Update Cargo.toml if different
CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -n1 | cut -d '"' -f2)
if [ "$CARGO_VERSION" != "$VERSION" ]; then
    log_info "Updating Cargo.toml version to ${VERSION}..."
    sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
    sed -i "s/const VERSION: &str = \".*\";/const VERSION: &str = \"${VERSION}\";/" src/main.rs
fi

# Check CHANGELOG.md has entry for this version
if ! grep -q "## \[${VERSION}\]" CHANGELOG.md; then
    log_warn "CHANGELOG.md does not appear to have an entry for [${VERSION}]."
    read -rp "Continue anyway? (y/N): " CONFIRM
    if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
        log_error "Release aborted. Please document changes in CHANGELOG.md."
    fi
fi

# Verify tests, formatting, and lints pass
log_info "Running test suite..."
cargo test --all-targets --quiet

log_info "Checking code formatting..."
cargo fmt --all --check

log_info "Running Clippy linter..."
cargo clippy --all-targets -- -D warnings

# Commit version bump if changes were made
if [ -n "$(git status --porcelain)" ]; then
    log_info "Committing version bump..."
    git add Cargo.toml src/main.rs CHANGELOG.md
    git commit -m "chore(release): bump version to ${VERSION}"
fi

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    log_error "Tag '${TAG}' already exists locally or remotely."
fi

# Create annotated tag
log_info "Creating git tag ${TAG}..."
git tag -a "$TAG" -m "Release ${TAG}"

log_success "Tag ${TAG} created successfully."

# Push to GitHub
read -rp "Push commit and tag '${TAG}' to origin/main to trigger release pipeline? (y/N): " PUSH_CONFIRM
if [[ "$PUSH_CONFIRM" =~ ^[Yy]$ ]]; then
    log_info "Pushing to GitHub..."
    git push origin main
    git push origin "$TAG"
    log_success "Pushed! GitHub Actions will now build and publish release ${TAG}."
    echo -e "Track release progress at: ${BLUE}https://github.com/imyash0722/mdview/actions${RESET}"
else
    log_warn "Tag '${TAG}' created locally. To publish manually, run:"
    echo "  git push origin main && git push origin ${TAG}"
fi
