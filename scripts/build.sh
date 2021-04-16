#!/bin/sh -e

SCRIPT_FILE=$(readlink -f $0)
SCRIPT_DIR=$(dirname "$SCRIPT_FILE")
ROOT_DIR=$(dirname "$SCRIPT_DIR")

cd "$ROOT_DIR"
VERSION=$(cat '.versionrc.json' | jq -r '.version')
VERSION_MAJOR=$(echo "$VERSION" | cut -d '.' -f 1)
VERSION_MINOR=$(echo "$VERSION" | cut -d '.' -f 2)
VERSION_SHORT="${VERSION_MAJOR}.${VERSION_MINOR}"
docker build . -f docker/bin.dockerfile -t "rhangai/auth-gatekeeper:bin" -t "rhangai/auth-gatekeeper:bin-${VERSION_SHORT}" -t "rhangai/auth-gatekeeper:bin-${VERSION_MAJOR}"
docker build . -f docker/dockerfile -t "rhangai/auth-gatekeeper" -t "rhangai/auth-gatekeeper:${VERSION_SHORT}" -t "rhangai/auth-gatekeeper:${VERSION_MAJOR}"

echo "Built tags"
echo "  rhangai/auth-gatekeeper:bin"
echo "  rhangai/auth-gatekeeper:bin-${VERSION_MAJOR}"
echo "  rhangai/auth-gatekeeper:bin-${VERSION_SHORT}"
echo "  rhangai/auth-gatekeeper"
echo "  rhangai/auth-gatekeeper:${VERSION_MAJOR}"
echo "  rhangai/auth-gatekeeper:${VERSION_SHORT}"