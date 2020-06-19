FROM node:14

# Add Tini
ENV TINI_VERSION v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini

WORKDIR /opt/auth-gatekeeper
ADD . /opt/auth-gatekeeper
RUN \
	yarn install && \
	yarn build && \
	yarn install --production && \
	yarn cache clean


ENV AUTH_GATEKEEPER_HOST=0.0.0.0
ENTRYPOINT ["/tini", "--", "node", "/opt/auth-gatekeeper/dist/index.js"]