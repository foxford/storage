# Object

## Read

Retrieve an object using its bucket and object names (through redirect to underlying storage).

**URI**

```
GET /buckets/${BUCKET}/objects/${OBJECT}
```

**URI parameters**

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
BUCKET | string | _required_ | Name of the bucket.
OBJECT | string | _required_ | Name of the object.

**Response**

Redirect to the object URI in the underlying storage (`303 "See Other"` status code).

**Example**

```bash
curl -fsSL \
    -XGET ${ENDPOINT}/buckets/example.org/objects/foo.bar \
    -H "authorization: Bearer ${ACCESS_TOKEN}"
```
