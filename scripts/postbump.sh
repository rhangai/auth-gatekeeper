#!/bin/sh -e

SCRIPT_FILE=$(readlink -f $0)
SCRIPT_DIR=$(dirname "$SCRIPT_FILE")
ROOT_DIR=$(dirname "$SCRIPT_DIR")

cd "$ROOT_DIR"
VERSION=$(cat package.json | jq -r ".version")
sed -i "s/^version\s*=.*$/version = \"$VERSION\"/" Cargo.toml
git add Cargo.toml