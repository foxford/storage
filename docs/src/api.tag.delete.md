# Delete

Delete the tag.

**URI**

```
DELETE /tags/${TAG}
```

**URI parameters**

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
TAG    | Tag    | _required_ | Alias, group, or category of the set.

**Response**

If successful, `204 "No Content"` status code is returned in response.

**Example**

```bash
curl -fsSL \
    -XDELETE ${ENDPOINT}/tags/ref.example.org::foo \
    -H "authorization: Bearer ${ACCESS_TOKEN}"
```