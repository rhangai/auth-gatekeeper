FROM traefik as traefik

FROM rust:1.49
WORKDIR /opt/auth-gatekeeper

RUN apt-get update && apt-get install -y curl supervisor
RUN mkdir -p /var/run/auth-gatekeeper
ENV AUTH_GATEKEEPER_LISTEN=http://127.0.0.1:8088/

COPY --from=traefik /usr/local/bin/traefik /usr/local/bin/traefik
COPY ./docker/traefik/supervisord.conf /etc/supervisor/conf.d/supervisord.conf
COPY ./docker/traefik/etc /etc/traefik

COPY ./docker/traefik/docker-entrypoint.sh /docker-entrypoint.sh
ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["traefik", "--configFile=/etc/traefik/traefik.debug.toml"]
EXPOSE 80