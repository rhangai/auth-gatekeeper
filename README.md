# auth-gatekeeper

Usage with docker-compose

```yml
version: '3.7'
services:
    proxy:
        image: rhangai/auth-gatekeeper:nginx
    environment:
        - AUTH_GATEKEEPER_SECRET=
        - AUTH_GATEKEEPER_PROVIDER=
        - AUTH_GATEKEEPER_PROVIDER_CLIENT_ID=
        - AUTH_GATEKEEPER_PROVIDER_CLIENT_SECRET=
        - AUTH_GATEKEEPER_PROVIDER_AUTH_URL=
        - AUTH_GATEKEEPER_PROVIDER_TOKEN_URL=
        - AUTH_GATEKEEPER_PROVIDER_USERINFO_URL=
        - AUTH_GATEKEEPER_PROVIDER_CALLBACK_URL=
        - |
            NGINX_DEFAULT_CONFIG=
            server {
              include auth-gatekeeper/server.conf;

              root /var/www/html/;
              index index.html;

              location /restrict/ {
                include auth-gatekeeper/auth-request.conf;
                error_page 401 = @auth-redirect;
              }
            }
```

## Configuration

-   `AUTH_GATEKEEPER_SECRET`: Secrets to encrypt the cookies (If not set, a random one will be used everytime invalidating every session)
-   `AUTH_GATEKEEPER_JWT_SECRET`: Secrets to encode the x-auth-userinfo header and the endpoints data.
-   `AUTH_GATEKEEPER_COOKIE_ACCESS_TOKEN_NAME`: Name of the cookie for the access token.
-   `AUTH_GATEKEEPER_COOKIE_REFRESH_TOKEN_NAME`: Name of the cookie for the refresh token.
-   `AUTH_GATEKEEPER_PROVIDER`: Provider for the gatekeeper. `oidc` or `keycloak`
-   `AUTH_GATEKEEPER_PROVIDER_CLIENT_ID`: ID of the openid client
-   `AUTH_GATEKEEPER_PROVIDER_CLIENT_SECRET`: Secret of the openid client
-   `AUTH_GATEKEEPER_PROVIDER_AUTH_URL`: Authorization endpoint
-   `AUTH_GATEKEEPER_PROVIDER_TOKEN_URL`: Token endpoint
-   `AUTH_GATEKEEPER_PROVIDER_USERINFO_URL`: Userinfo endpoint (Not used in keycloak mode)
-   `AUTH_GATEKEEPER_PROVIDER_CALLBACK_URL`: Callback url
-   `NGINX_DEFAULT_CONFIG`: Nginx configuration to use
