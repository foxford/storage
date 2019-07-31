## Read

Retrieve an object with specified set and name (through redirect to underlying storage).

**URI**

```
GET /sets/${SET}/objects/${OBJECT}
```

**URI parameters**

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
SET    | Set    | _required_ | Location on the underlying backend.
OBJECT | String | _required_ | Name of the object.

**Response**

Redirect to the object URI in the underlying storage (`303 "See Other"` status code).

**Example**

```bash
curl -fsSL \
    -XGET ${ENDPOINT}/sets/data.example.org:foo/objects/bar \
    -H "authorization: Bearer ${ACCESS_TOKEN}"
```
