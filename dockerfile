FROM node:14 

WORKDIR /opt/auth-proxy
ADD . /opt/auth-proxy
RUN yarn install && yarn build

ENTRYPOINT ["node", "/opt/auth-proxy/dist/index.js"]