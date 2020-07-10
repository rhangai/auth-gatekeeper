# auth-gatekeeper

Usage with docker-compose

```yml
version: '3.7'
services:
    proxy:
        image: rhangai/auth-gatekeeper:traefik
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
            TRAEFIK_CONFIG=
            [http.routers]
              [http.routers.backend]
                rule = "PathPrefix(`/api/`)"
                middlewares = ["auth"]
                service = "backend"
              [http.routers.frontend]
                rule = "PathPrefix(`/`)"
                priority = 1
                middlewares = ["auth-redirect"]
                service = "frontend"

            [http.services]
              [http.services.backend.loadBalancer]
                [[http.services.backend.loadBalancer.servers]]
                  url = "http://some-backend-server/"
              [http.services.frontend.loadBalancer]
                [[http.services.frontend.loadBalancer.servers]]
                  url = "http://some-frontend-ip/"
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
-   `TRAEFIK_DEBUG`: Set to `1` to enable debug
-   `TRAEFIK_CONFIG`: Traefik configuration to use

## Traefik config

When using traefik, some pre-defined config are placed on `/etc/traefik/providers/auth.toml` file inside the container.

### Middlewares

-   `auth`: Authenticate only, and returns a 401 if not authorized
-   `auth-redirect`: Authenticate, and send the user to the login page

### Routes

-   `/login?url=`: Login the user and redirects it to the page
-   `/logout`: Logout the user
-   `/auth/callback`: Callback for the oauth
-   `/auth/refresh`: Refresh the session, and returns the userdata. Useful to get user info when logged.

### Services

-   `auth`: The authentication service inside the container
