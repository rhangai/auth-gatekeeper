# auth-gatekeeper

Usage with docker-compose

```yml
version: '3.7'
services:
    proxy:
        image: nginx
```

## Configuration

-   `AUTH_GATEKEEPER_SECRET`: Secrets to encrypt the cookies (If not set, a random one will be used everytime invalidating every session)
-   `AUTH_GATEKEEPER_PROVIDER`: Provider for the gatekeeper. `oidc` or `keycloak`
