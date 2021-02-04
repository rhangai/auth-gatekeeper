FROM traefik as traefik

FROM rust:1.49
WORKDIR /opt/auth-gatekeeper

RUN apt-get update && apt-get install -y openssl
COPY --from=traefik /usr/local/bin/traefik /usr/local/bin/traefik
ADD ./docker/etc /etc
COPY ./docker/scripts/docker-entrypoint.sh /docker-entrypoint.sh

ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["traefik", "--configFile=/etc/traefik/traefik.debug.toml"]
EXPOSE 80

# Configurações padrões
ENV AUTH_GATEKEEPER_LISTEN=http://127.0.0.1:8088/