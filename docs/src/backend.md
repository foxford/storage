# Backend

**S3-compatible** underlying storage.


## Data representation

### Object API

```bash
buckets/${BUCKET}/objects/${OBJECT}
```

Bucket        | Object
------------- | --------
`BUCKET`      | `OBJECT`



### Set API

```bash
buckets/${BUCKET}/sets/${SET}/objects/${OBJECT}
```

Bucket        | Object
------------- | --------------
`BUCKET`      | `SET`.`OBJECT`
