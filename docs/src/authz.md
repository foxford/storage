# Authorization

In order to authorize an **action** performed by a **subject** to an **object**, the application sends a `POST` request to the authorization endpoint.

**Example**

```json
{
    "subject": {
        "namespace": "iam.example.org",
        "value": ["accounts", "123e4567-e89b-12d3-a456-426655440000"]
    },
    "object": {
        "namespace": "storage.example.org",
        "value": ["buckets", "foo", "sets", "bar"]
    },
    "action": "read"
}
```

Subject's namespace and account identifier are retrieved from `aud` and `sub` claims of an **access token** respectively. If an access token is not presented in a request, the account identifier will be replaced with `"anonymous"` keyword. URI of authorization endpoint, object and anonymous namespaces are configured through the application configuration file.

Possible values for `SUBJECT`:
- `["accounts", ACCOUNT_ID]`
- `["accounts", "anonymous"]`

Possible values for `OBJECT`:
- `["buckets", BUCKET, "sets", SET]`
- `["buckets", BUCKET, "objects", OBJECT]`

Possible values for `ACTION`:
- `"create"`
- `"read"`
- `"update"`
- `"delete"`