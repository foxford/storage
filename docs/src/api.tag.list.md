# List

Retrieve a list of tags.

**URI**

```
GET /tags
```

**Query string parameters**

Name     | Type   | Default    | Description
-------- | ------ | ---------- | ------------------
filter   | Bucket | _required_ | Returns only tags of the specified type.
include  | [Tag]  | _required_ | Queried sets should have tags from this list.
exclude  | [Tag]  | _optional_ | Queried sets shouldn't have tags from this list.
offset   | Int    | _optional_ | Returns only objects starting from the specified index.
limit    | Int    |         25 | Limits the number of objects in the response.

**Response**

If successful, the response contains a list of `Tag` objects.

**Example**

```bash
curl -fsSL \
    -XGET ${ENDPOINT}/tags?filter=ref.example.org&include=group.example.org:a,group.example.org:b&exclude=group.example.org:c \
    -H "authorization: Bearer ${ACCESS_TOKEN}"

[
    "ref.example.org:foo"
]
```
