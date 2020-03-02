# Authorization

In order to authorize an **action** performed by a **subject** to an **object**, the application sends a `POST` request to the authorization endpoint.

**Example**

```json
{
    "subject": {
        "namespace": "iam.example.org",
        "value": "123e4567-e89b-12d3-a456-426655440000"
    },
    "object": {
        "namespace": "storage.example.org",
<<<<<<< HEAD
        "value": ["buckets", "foo", "sets", "bar"]
=======
        "value": ["sets", "data.example.org::foo"]
>>>>>>> e53f9c9... Change the set separator
    },
    "action": "read"
}
```

Subject's namespace and account label are retrieved from `aud` and `sub` claims of an **access token** respectively. If the access token is not presented in a request, the `"anonymous"` keyword will be sent as account label. URI of authorization endpoint, object and anonymous namespaces are configured through the application configuration file.

Possible values for `SUBJECT`:

subject                     |
--------------------------- | -
["accounts", ACCOUNT_LABEL] |
["accounts", "anonymous"]   |

Possible values for `OBJECT` and `ACTION`:

object / action                        | read | update | delete | list
-------------------------------------- | ---- | ------ | ------ | ----
["sets", SET]                          |    + |      + |      + | -
["tags", TAG]                          |    + |      + |      + | -
["tags"]                               |    - |      - |      - | +

Note that `SET` and `TAG` must contain the audience of the tenant the request will be sent to. For example, for the sets `data.example.org:foo` and `data.example.org:foo` requests will be sent to the `example.org` audience (the audience should be presented in the application configuration).
