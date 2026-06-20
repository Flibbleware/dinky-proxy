#!/usr/bin/env bash
set -euo pipefail

CARGO_TOML="apps/ui/src-tauri/Cargo.toml"

usage() {
  echo "Usage: $0 <version> [--push]"
  echo "  version   Semver string, e.g. 1.0.0"
  echo "  --push    Push commit and tag to origin after tagging"
  exit 1
}

# ── Args ──────────────────────────────────────────────────────────────────────

VERSION=""
PUSH=false

for arg in "$@"; do
  case "$arg" in
    --push) PUSH=true ;;
    --*)    echo "Unknown option: $arg"; usage ;;
    *)      VERSION="$arg" ;;
  esac
done

[[ -z "$VERSION" ]] && usage

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: '$VERSION' is not a valid semver (expected X.Y.Z)"
  exit 1
fi

TAG="v$VERSION"

# ── Preflight ─────────────────────────────────────────────────────────────────

cd "$(git rev-parse --show-toplevel)"

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "error: working tree has uncommitted changes — commit or stash them first"
  exit 1
fi

if git rev-parse "$TAG" &>/dev/null; then
  echo "error: tag '$TAG' already exists"
  exit 1
fi

CURRENT=$(sed -n 's/^version = "\(.*\)"/\1/p' "$CARGO_TOML" | head -n1)
echo "Bumping $CURRENT → $VERSION"

# ── Update Cargo.toml ─────────────────────────────────────────────────────────

# Only replace the first occurrence (the [package] version, not dependency versions)
awk -v new="$VERSION" '
  !done && /^version = "/ { sub(/"[^"]*"/, "\"" new "\""); done=1 }
  { print }
' "$CARGO_TOML" > "$CARGO_TOML.tmp" && mv "$CARGO_TOML.tmp" "$CARGO_TOML"

# Sync the lock file so it doesn't end up dirty after tagging
cargo update --manifest-path "$CARGO_TOML" -p dinkyproxy-ui --precise "$VERSION"

# ── Commit & tag ──────────────────────────────────────────────────────────────

git add "$CARGO_TOML" "apps/ui/src-tauri/Cargo.lock"
git commit -m "chore: release $TAG"
git tag "$TAG"

echo "Created commit and tag '$TAG'"

# ── Push (opt-in) ─────────────────────────────────────────────────────────────

if $PUSH; then
  git push origin HEAD
  git push origin "$TAG"
  echo "Pushed to origin — CI build is running"
else
  echo ""
  echo "Run the following to trigger CI:"
  echo "  git push origin HEAD && git push origin $TAG"
fi
