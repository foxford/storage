# Read

Retrieve the first object with specified tag and name (through redirect to underlying storage).

**URI**

```
GET /tags/${TAG}/objects/${OBJECT}
```

**URI parameters**

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
TAG    | Tag    | _required_ | Alias, group, or category of the set.
OBJECT | String | _required_ | Name of the object.

**Response**

Redirect to the object URI in the underlying storage (`303 "See Other"` status code).

**Example**

```bash
curl -fsSL \
    -XGET ${ENDPOINT}/tags/ref.example.org::foo/objects/bar \
    -H "authorization: Bearer ${ACCESS_TOKEN}"
```
