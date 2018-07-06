# Set

## Read

Returns an object specified by bucket, set and key (through redirect to underlying storage).

**URI**

```
GET /buckets/${BUCKET}/sets/${SET}/objects/${KEY}
```

**URI parameters**

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
BUCKET | string | _required_ | Name of the bucket
SET    | string | _required_ | Name of the set
KEY    | string | _required_ | Key of the object

**Response**

Redirect to the object URI in the underlying storage (307 status code)

**Example**

```bash
curl -fsSL \
    -XGET ${ENDPOINT}/buckets/example-bucket/objects/example \
    -H "authorization: Bearer ${ACCESS_TOKEN}"
```



## Authorization

**ABAC attributes**

Name         | Value
------------ | ------
namespace_id | `STORAGE_NAMESPACE_ID`
key          | uri
value        | `BUCKET`/`SET`



## Data representation

Mapping of object URI:

```bash
buckets/${BUCKET}/sets/${SET}/objects/${KEY}
```

to **S3-compatible** underlying storage would be:

Bucket        | Object
------------- | ------
`BUCKET`      | `SET`.`KEY`
