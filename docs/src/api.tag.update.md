## Update

Update the tag for the specified set.

**URI**

```
PUT /tags/${TAG}
```

**URI parameters**

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
TAG    | Tag    | _required_ | Alias, group, or category of the set.

**Payload**

Name         | Type   | Default    | Description
------------ | ------ | ---------- | ------------------
set          | Set    | _required_ | Location on the underlying backend.

**Response**

If successful, `204 "No Content"` status code is returned in response.

**Example**

```bash
curl -fsSL \
    -XPUT ${ENDPOINT}/tags/ref.example.org::foo \
    -H "authorization: Bearer ${ACCESS_TOKEN}" \
    -H 'content-type: application/json' \
    --data-binary '{"set": "data.example.org::foo"}'
```
