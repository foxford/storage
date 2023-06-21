# Sign

Retrieve a signed URI of content in the underlying storage.

**URI**

```
POST /backends/${BACKEND}/sign
```

**URI parameters**

| Name    | Type   | Default    | Description         |
|---------|--------|------------|---------------------|
| BACKEND | String | _required_ | Name of the backend |

**Payload**

| Name       | Type   | Default    | Description                                                                               |
|------------|--------|------------|-------------------------------------------------------------------------------------------|
| set        | Set    | _required_ | Location on the underlying backend.                                                       |
| object     | String | _required_ | Name of the object.                                                                       |
| method     | String | _required_ | HTTP Method of the actual request, could be one of these: `HEAD`, `GET`, `PUT`, `DELETE`. |
| headers    | Object | _required_ | HTTP Headers of the actual request, `content-type` is required.                           |
| expires_in | Int    | 300        | Expiration time requested for a signature of the actual request.                          |

**Response**

| Name | Type   | Default    | Description                           |
|------|--------|------------|---------------------------------------|
| uri  | String | _required_ | Signed URI of the underlying storage. |

**Example**

```bash
curl -fsSL \
    -X POST "${ENDPOINT}/backends/${BACKEND}/sign" \
    -H "authorization: Bearer ${ACCESS_TOKEN}" \
    -H 'content-type: application/json' \
    --data-binary '{"set": "data.example.org::foo", "object": "bar", "method": "PUT", "headers": {"content-type": "text/plain"}}'

{
  "uri": "https://s3.example.org/example.org/foo.bar?AWSAccessKeyId=7HAbGrmLzeWa4T8R&Expires=1530820731&Signature=bnIwiFU1iqlR7PdWnelPHkvjnKE%3D"
}
```
