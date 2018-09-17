# Authorization

In order to authorize an **action** performed by a **subject** to an **object**, the application sends a `POST` request to the `AUTHZ_ENDPOINT`.

```json
{
    "subject": {
        "namespace": SUBJECT_NAMESPACE,
        "value": SUBJECT
    },
    "object": {
        "namespace": OBJECT_NAMESPACE,
        "value": OBJECT
    },
    "action": ACTION
}
```

Where
- `SUBJECT_NAMESPACE` and `SUBJECT` are retrieved from `aud` and `sub` claims of an **access token** respectively. If the access token is not presented in the request, `{"namespace": SUBJECT_NAMESPACE, "value": ["accounts", "anonymous"]}` is used.
- `AUTHZ_ENDPOINT`, `SUBJECT_NAMESPACE` and `OBJECT_NAMESPACE` are specified in the application config file under `authz` key.

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