
#!/bin/bash
# Version management script for memos-rs
# Usage: ./scripts/bump.sh [major|minor|patch|patchrc|version X.Y.Z[RC{num}]]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Read current version
CURRENT_VERSION=$(grep '^version = ' "$PROJECT_DIR/Cargo.toml" | sed 's/version = "//;s/"//')
echo "Current version: $CURRENT_VERSION"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
RC_NUMBER=""

# Check if it's an RC version
if [[ "$PATCH" =~ RC([0-9]+)$ ]]; then
    RC_NUMBER="${BASH_REMATCH[1]}"
    PATCH="${PATCH%RC*}"  # Remove RC suffix from PATCH for proper incrementing
fi

# Handle version bumping based on argument
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
    patchrc)
        if [[ -z "$RC_NUMBER" ]]; then
            RC_NUMBER=1
        else
            RC_NUMBER=$((RC_NUMBER + 1))
        fi
        PATCHRC="$PATCH"
        ;;
    version)
        if [ -z "${2:-}" ]; then
            echo "Error: Version number required"
            echo "Usage: $0 version X.Y.Z[RC{num}]"
            exit 1
        fi
        NEW_VERSION="${2}"

        # Match X.Y.Z or X.Y.ZRCN formats
        if [[ ! "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] && [[ ! "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+RC([0-9]+)$ ]]; then
            echo "Error: Version must be in format X.Y.Z or X.Y.ZRCN"
            exit 1
        fi

        IFS='.' read -r MAJOR MINOR PATCHRC <<< "$NEW_VERSION"
        # Extract the RC number if present
        if [[ "$PATCHRC" =~ RC([0-9]+)$ ]]; then
            RC_NUMBER="${BASH_REMATCH[1]}"
            PATCHRC="${PATCHRC%RC*}"
        else
            RC_NUMBER=""
	    PATCH=$PATCHRC
        fi
        ;;
    *)
        echo "Usage: $0 [major|minor|patch|patchrc|version X.Y.Z[RC{num}] ]"
        echo ""
        echo "Examples:"
        echo "  $0 patch      # Bump patch version (0.1.0 -> 0.1.1)"
        echo "  $0 patchrc   # Bump patch-rc version (0.1.0RC1 -> 0.1.0RC2)"
        echo "  $0 minor      # Bump minor version (0.1.0 -> 0.2.0)"
        echo "  $0 major      # Bump major version (0.1.0 -> 1.0.0)"
        echo "  $0 version 0.2.0       # Set specific version"
        echo "  $0 version 0.2.0RC1    # Set specific pre-release version"
        exit 1
        ;;
esac

# Construct new version string
if [[ -n "$RC_NUMBER" ]]; then
    echo "RC version is present"
    NEW_VERSION="$MAJOR.$MINOR.$PATCHRC"-rc."$RC_NUMBER"  # For RC versions (e.g. 1.2.3RC9)
else
    NEW_VERSION="$MAJOR.$MINOR.$PATCH"  # For regular versions
fi

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

if [[ "$NEW_VERSION" =~ RC([0-9]+) ]]; then
    # For RC versions, check if the base version exists in the changelog
    BASE_VERSION="$MAJOR.$MINOR.$PATCHRC"
    if grep -q "\[$BASE_VERSION\]" "$PROJECT_DIR/CHANGELOG.md"; then
        # Replace the Unreleased header and add the new RC version header
        sed -i "s/$UNRELEASED_HEADER/$NEW_VERSION_HEADER\n\n### Added\n\n### Changed\n\n### Fixed\n\n$UNRELEASED_HEADER/" "$PROJECT_DIR/CHANGELOG.md"
        echo "  CHANGELOG.md: Updated from $BASE_VERSION to $NEW_VERSION"
    else
        echo "  CHANGELOG.md: No matching version section found"
    fi
else
    # For regular versions
    if grep -q "$UNRELEASED_HEADER" "$PROJECT_DIR/CHANGELOG.md"; then
        sed -i "s/$UNRELEASED_HEADER/$NEW_VERSION_HEADER\n\n### Added\n\n### Changed\n\n### Fixed\n\n$UNRELEASED_HEADER/" "$PROJECT_DIR/CHANGELOG.md"
        echo "  CHANGELOG.md: Added $NEW_VERSION header"
    else
        echo "  CHANGELOG.md: No unreleased section found"
    fi
fi

echo ""
echo "To complete the release:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git commit -am 'Release $NEW_VERSION'"
echo "  3. Tag: git tag -a v$NEW_VERSION -m 'Release $NEW_VERSION'"
echo "  4. Push: git push && git push origin v$NEW_VERSION"
