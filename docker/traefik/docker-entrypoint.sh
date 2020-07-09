#!/bin/sh

if [ -n "$TRAEFIK_CONFIG" ]; then
	echo "$TRAEFIK_CONFIG" > /etc/traefik/providers/default.toml
fi
exec "$@"