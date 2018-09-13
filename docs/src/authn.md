# Authentication

In order to authenticate requests, **access tokens** in form of **JSON Web Tokens (JWT)** are used. A valid access token must contain `sub` and `aud` claims. Other claims are optional.

Each identity provider must be specified in the application config file under `authn` key.
