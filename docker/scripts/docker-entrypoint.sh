#!/bin/sh

mkdir -p /etc/traefik/providers
if [ "$TRAEFIK_DEBUG" = "1" ]; then
	cp /etc/traefik/traefik.debug.toml /etc/traefik/traefik.toml
else
	cp /etc/traefik/traefik.prod.toml /etc/traefik/traefik.toml
fi
if [ -n "$TRAEFIK_CONFIG" ]; then
	echo "$TRAEFIK_CONFIG" > /etc/traefik/providers/default.toml
fi
dockerize -template  /etc/traefik/template/auth.toml:/etc/traefik/providers/auth.toml
exec "$@"