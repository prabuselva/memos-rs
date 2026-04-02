#!/bin/bash

# Version management script for memos-rs
# Usage: ./scripts/bump.sh [major|minor|patch|version X.Y.Z]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Read current version
CURRENT_VERSION=$(grep '^version = ' "$PROJECT_DIR/Cargo.toml" | sed 's/version = "//;s/"//')

echo "Current version: $CURRENT_VERSION"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

case "${1:-}" in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
    version)
        if [ -z "${2}" ]; then
            echo "Error: Version number required"
            echo "Usage: $0 version X.Y.Z"
            exit 1
        fi
        NEW_VERSION="${2}"
        if [[ ! "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            echo "Error: Version must be in format X.Y.Z"
            exit 1
        fi
        IFS='.' read -r MAJOR MINOR PATCH <<< "$NEW_VERSION"
        ;;
    *)
        echo "Usage: $0 [major|minor|patch|version X.Y.Z]"
        echo ""
        echo "Examples:"
        echo "  $0 patch      # Bump patch version (0.1.0 -> 0.1.1)"
        echo "  $0 minor      # Bump minor version (0.1.0 -> 0.2.0)"
        echo "  $0 major      # Bump major version (0.1.0 -> 1.0.0)"
        echo "  $0 version 0.2.0  # Set specific version"
        exit 1
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

echo "New version: $NEW_VERSION"

# Update Cargo.toml (only the package version, not dependency versions)
sed -i "0,/^version = /s/^version = .*/version = \"$NEW_VERSION\"/" "$PROJECT_DIR/Cargo.toml"

# Update frontend/package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" "$PROJECT_DIR/frontend/package.json"

echo "Updated versions:"
echo "  Cargo.toml: $NEW_VERSION"
echo "  frontend/package.json: $NEW_VERSION"

# Update CHANGELOG.md with unreleased section
UNRELEASED_HEADER="## [Unreleased]"
NEW_VERSION_HEADER="## [$NEW_VERSION] - $(date +%Y-%m-%d)"

if grep -q "$UNRELEASED_HEADER" "$PROJECT_DIR/CHANGELOG.md"; then
    sed -i "s/$UNRELEASED_HEADER/$NEW_VERSION_HEADER\n\n### Added\n\n### Changed\n\n### Fixed\n\n$UNRELEASED_HEADER/" "$PROJECT_DIR/CHANGELOG.md"
    echo "  CHANGELOG.md: Added $NEW_VERSION header"
else
    echo "  CHANGELOG.md: No unreleased section found"
fi

echo ""
echo "To complete the release:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git commit -am 'Release $NEW_VERSION'"
echo "  3. Tag: git tag -a v$NEW_VERSION -m 'Release $NEW_VERSION'"
echo "  4. Push: git push && git push origin v$NEW_VERSION"