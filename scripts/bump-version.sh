#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/bump-version.sh <patch|minor|major>
#
# Bumps version for all crates in the workspace and syncs inter-crate dependency versions.
# All 3 crates are kept at the same version.

BUMP_TYPE="${1:-}"

if [[ -z "$BUMP_TYPE" ]]; then
  echo "Usage: $0 <patch|minor|major>"
  exit 1
fi

if [[ "$BUMP_TYPE" != "patch" && "$BUMP_TYPE" != "minor" && "$BUMP_TYPE" != "major" ]]; then
  echo "Error: bump type must be one of: patch, minor, major"
  exit 1
fi

# --- Resolve current version from root Cargo.toml ---
CURRENT_VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

if [[ -z "$CURRENT_VERSION" ]]; then
  echo "Error: could not read current version from Cargo.toml"
  exit 1
fi

IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

case "$BUMP_TYPE" in
  major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
  minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
  patch) PATCH=$((PATCH + 1)) ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"

echo "Bumping version: $CURRENT_VERSION -> $NEW_VERSION"

sedi() {
  if [[ "$(uname)" == "Darwin" ]]; then
    sed -i '' "$@"
  else
    sed -i "$@"
  fi
}

update_package_version() {
  local file="$1" old="$2" new="$3"
  sedi "s/^version = \"${old}\"/version = \"${new}\"/" "$file"
  echo "  Updated package version in $file"
}

update_dep_version() {
  local file="$1" dep_name="$2" old="$3" new="$4"
  sedi "s/\(${dep_name} = {.*version = \"\)${old}\"/\1${new}\"/" "$file"
  echo "  Updated ${dep_name} dependency in $file"
}

# --- 1. sea-query-spanner ---
echo ""
echo "[sea-query-spanner]"
update_package_version "sea-query-spanner/Cargo.toml" "$CURRENT_VERSION" "$NEW_VERSION"

# --- 2. sea-orm-spanner (root) ---
echo ""
echo "[sea-orm-spanner]"
update_package_version "Cargo.toml" "$CURRENT_VERSION" "$NEW_VERSION"
update_dep_version "Cargo.toml" "sea-query-spanner" "$CURRENT_VERSION" "$NEW_VERSION"

# --- 3. sea-orm-migration-spanner ---
echo ""
echo "[sea-orm-migration-spanner]"
update_package_version "sea-orm-migration-spanner/Cargo.toml" "$CURRENT_VERSION" "$NEW_VERSION"
update_dep_version "sea-orm-migration-spanner/Cargo.toml" "sea-orm-spanner" "$CURRENT_VERSION" "$NEW_VERSION"
update_dep_version "sea-orm-migration-spanner/Cargo.toml" "sea-query-spanner" "$CURRENT_VERSION" "$NEW_VERSION"

echo ""
echo "Done! All crates bumped to $NEW_VERSION"
echo ""
echo "Verify with:"
echo "  grep -r '^version' Cargo.toml sea-query-spanner/Cargo.toml sea-orm-migration-spanner/Cargo.toml"
