# Sign

Retrieve a signed URI of content in the underlying storage.

**URI**

```
POST /sign
```

**Payload**

Name       | Type   | Default    | Description
---------- | ------ | ---------- | ------------------
bucket     | string | _required_ | Name of the bucket.
set        | string | _optional_ | Name of the set (if not specified, Object API is used).
object     | string | _required_ | Name of the object.
method     | string | _required_ | HTTP Method of the actual request, could be one of these: `HEAD`, `GET`, `PUT`, `DELETE`.
headers    | object | _required_ | HTTP Headers of the actual request, `content-type` is required.
expires_in | int    |        300 | Expiration time requested for a signature of the actual request.

**Response**

Name    | Type   | Default    | Description
------- | ------ | ---------- | ------------------
uri     | string | _required_ | Signed URI of the underlying storage.

**Example**

```bash
curl -fsSL \
    -X POST "https://storage.netology-group.services/api/v1/sign" \
    -H "authorization: Bearer ${ACCESS_TOKEN}" \
    -H 'content-type: application/json' \
    --data-binary '{"bucket": "example.org", "set": "foo", "object": "bar", "method": "PUT", "headers": {"content-type": "text/plain"}}'

{
  "uri": "https://s3.example.org/exampleorg/foo.bar?AWSAccessKeyId=7HAbGrmLzeWa4T8R&Expires=1530820731&Signature=bnIwiFU1iqlR7PdWnelPHkvjnKE%3D"
}
```
