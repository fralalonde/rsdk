#!/usr/bin/env bash
# Automates semantic version bumping and tagging in Git.
#
# Usage:
#   ./release.sh <major|minor|patch> [--push]
#
# Examples:
#   ./release.sh patch
#   ./release.sh minor --push
#
# Without --push, the script prompts before pushing. The default answer is no.
set -euo pipefail

show_help() {
    cat <<'EOF'
Automates semantic version bumping and tagging in Git.

Usage:
  release.sh <major|minor|patch> [--push]

Examples:
  ./release.sh patch
  ./release.sh minor --push

Without --push, the script prompts before pushing. The default answer is no.
EOF
}

increment=""
push=0
for arg in "$@"; do
    case "$arg" in
        major|minor|patch) increment="$arg" ;;
        --push) push=1 ;;
        -h|--help) show_help; exit 0 ;;
        *) echo "Unknown argument: $arg" >&2; show_help; exit 1 ;;
    esac
done

if [[ -z "$increment" ]]; then
    show_help
    exit 1
fi

# Get latest tag or default to v0.0.0
latest_tag="$(git tag --list 'v[0-9]*.[0-9]*.[0-9]*' --sort=-version:refname | head -n 1)"
latest_tag="${latest_tag:-v0.0.0}"

# Bump version
version_str="${latest_tag#v}"
IFS=. read -r major minor patch <<<"$version_str"
case "$increment" in
    major) major=$((major + 1)); minor=0; patch=0 ;;
    minor) minor=$((minor + 1)); patch=0 ;;
    patch) patch=$((patch + 1)) ;;
esac

new_tag="v${major}.${minor}.${patch}"
commit_message="Release $new_tag"

branch="$(git branch --show-current)"
if [[ -z "$branch" ]]; then
    echo "Aborted: repository is in detached HEAD state. Check out a branch before releasing." >&2
    exit 1
fi

# Check working directory
if [[ -n "$(git status --porcelain)" ]]; then
    echo "Repository has uncommitted changes:" >&2
    git status
    read -r -p "Add and commit all changes with message '$commit_message'? (y/N) " choice
    if [[ "$choice" =~ ^(y|yes)$ ]]; then
        git add -A
        git commit -m "$commit_message"
    else
        echo "Aborted due to uncommitted changes." >&2
        exit 1
    fi
fi

# Sync Cargo.toml's version to the release (git is the source of truth; the
# build file just carries it for the toolchain). Committed as part of the
# release if it changed.
new_version="${major}.${minor}.${patch}"
if [[ -f Cargo.toml ]] && ! grep -qE "^version = \"${new_version}\"" Cargo.toml; then
    sed -i -E "0,/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/s//version = \"${new_version}\"/" Cargo.toml
    # Keep the lockfile's own-package entry in sync (a metadata read rewrites it
    # without touching dependency pins, unlike `cargo update`).
    cargo metadata --format-version 1 >/dev/null 2>&1 || true
    git add Cargo.toml
    [[ -f Cargo.lock ]] && git add Cargo.lock
    git commit -m "$commit_message"
fi

# Tag
git tag -a "$new_tag" -m "$commit_message"
echo "Tagged with $new_tag"

push_release() {
    git push origin "$branch"
    git push origin "$new_tag"
    echo "Pushed branch $branch and tag $new_tag"
}

if [[ "$push" -eq 1 ]]; then
    push_release
else
    read -r -p "Push branch $branch and tag $new_tag to trigger the release pipeline? (y/N) " choice
    if [[ "$choice" =~ ^(y|yes)$ ]]; then
        push_release
    else
        echo "Not pushed. To trigger the release pipeline manually, run:"
        echo "  git push origin $branch"
        echo "  git push origin $new_tag"
    fi
fi
