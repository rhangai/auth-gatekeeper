#!/bin/sh

if [ -n "$NGINX_DEFAULT_CONFIG" ]; then
	echo "$NGINX_DEFAULT_CONFIG" > /etc/nginx/conf.d/default.conf
fi
exec "$@"