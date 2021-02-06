#!/bin/sh -e

SCRIPT_FILE=$(readlink -f $0)
SCRIPT_DIR=$(dirname "$SCRIPT_FILE")
ROOT_DIR=$(dirname "$SCRIPT_DIR")


cd "$ROOT_DIR"
VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r ".packages[0].version")
docker build . -f docker/bin.dockerfile -t "rhangai/auth-gatekeeper:bin" -t "rhangai/auth-gatekeeper:bin-${VERSION}"
docker build . -f docker/dockerfile -t "rhangai/auth-gatekeeper" -t "rhangai/auth-gatekeeper:${VERSION}"

echo "Built tags"
echo "  rhangai/auth-gatekeeper:bin"
echo "  rhangai/auth-gatekeeper:bin-${VERSION}"
echo "  rhangai/auth-gatekeeper"
echo "  rhangai/auth-gatekeeper:${VERSION}"