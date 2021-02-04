#!/bin/sh

SCRIPT_FILE=$(readlink -f $0)
SCRIPT_DIR=$(dirname "$SCRIPT_FILE")
ROOT_DIR=$(dirname "$SCRIPT_DIR")

cd "$ROOT_DIR"
docker build . -f docker/bin.dockerfile -t "rhangai/auth-gatekeeper:bin"
docker build . -f docker/dockerfile -t "rhangai/auth-gatekeeper"