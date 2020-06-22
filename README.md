# auth-gatekeeper

Usage with docker-compose

```yml
version: '3.7'
services:
    proxy:
        image: nginx
```

## Configuration

-   `AUTH_GATEKEEPER_PROVIDER`: Provider for the gatekeeper. `oidc` or `keycloak`
-   `AUTH*GATEKEEPER_PROVIDER*
