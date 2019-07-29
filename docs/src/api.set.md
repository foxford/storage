# Set

## Read

Retrieve an object using its bucket, set and object names (through redirect to underlying storage).

**URI**

```
GET /buckets/${BUCKET}/sets/${SET}/objects/${OBJECT}
```

**URI parameters**

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
BUCKET | string | _required_ | Name of the bucket.
SET    | string | _required_ | Name of the set.
OBJECT | string | _required_ | Name of the object.

**Response**

Redirect to the object URI in the underlying storage (`303 "See Other"` status code).

**Example**

```bash
curl -fsSL \
    -XGET ${ENDPOINT}/buckets/example.org/sets/foo/objects/bar \
    -H "authorization: Bearer ${ACCESS_TOKEN}"
```
