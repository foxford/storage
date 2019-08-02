use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::result::Error;
use uuid::Uuid;

use crate::db::{Bucket, Set};
use crate::schema::set_tag;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Identifiable, Queryable, QueryableByName)]
#[table_name = "set_tag"]
pub(crate) struct Object {
    id: Uuid,
    tag: Set,
    set: Set,
    created_at: DateTime<Utc>,
}

impl Object {
    pub(crate) fn set(&self) -> &Set {
        &self.set
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) struct FindQuery<'a> {
    id: Option<Uuid>,
    tag: Option<&'a Set>,
}

impl<'a> FindQuery<'a> {
    pub(crate) fn new() -> Self {
        Self {
            id: None,
            tag: None,
        }
    }

    pub(crate) fn tag(mut self, tag: &'a Set) -> Self {
        self.tag = Some(tag);
        self
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<Option<Object>, Error> {
        use diesel::prelude::*;

        match (self.id, self.tag) {
            (Some(id), _) => set_tag::table.find(id).get_result(conn).optional(),
            (_, Some(tag)) => set_tag::table
                .filter(set_tag::tag.eq(tag))
                .order_by(set_tag::created_at.asc())
                .get_result(conn)
                .optional(),
            _ => Err(Error::QueryBuilderError(
                "id or tag is required parameter of the query".into(),
            )),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) struct DeleteQuery<'a> {
    id: Option<Uuid>,
    tag: Option<&'a Set>,
}

impl<'a> DeleteQuery<'a> {
    pub(crate) fn new() -> Self {
        Self {
            id: None,
            tag: None,
        }
    }

    pub(crate) fn tag(self, tag: &'a Set) -> Self {
        Self {
            tag: Some(tag),
            ..self
        }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<usize, Error> {
        use diesel::prelude::*;

        match (self.id, self.tag) {
            (Some(id), _) => {
                diesel::delete(set_tag::table.filter(set_tag::id.eq(id))).execute(conn)
            }
            (_, Some(tag)) => {
                diesel::delete(set_tag::table.filter(set_tag::tag.eq(tag))).execute(conn)
            }
            _ => Err(Error::QueryBuilderError(
                "id or tag is required parameter of the query".into(),
            )),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "set_tag"]
pub(crate) struct UpdateQuery<'a> {
    id: Option<Uuid>,
    tag: &'a Set,
    set: &'a Set,
}

impl<'a> UpdateQuery<'a> {
    pub(crate) fn new(tag: &'a Set, set: &'a Set) -> Self {
        Self { id: None, tag, set }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<Object, Error> {
        use crate::schema::set_tag::dsl::set_tag;
        use diesel::RunQueryDsl;

        diesel::insert_into(set_tag)
            .values(self)
            .on_conflict((crate::schema::set_tag::tag, crate::schema::set_tag::set))
            .do_update()
            .set(self)
            .get_result(conn)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) struct ListQuery<'a> {
    kind: &'a Bucket,
    include: Vec<Set>,
    exclude: Vec<Set>,
    limit: i64,
    offset: i64,
}

impl<'a> ListQuery<'a> {
    pub(crate) fn new(
        kind: &'a Bucket,
        include: Vec<Set>,
        exclude: Vec<Set>,
        limit: i64,
        offset: i64,
    ) -> Self {
        Self {
            kind,
            include,
            exclude,
            limit,
            offset,
        }
    }

    pub(crate) fn execute(&self, conn: &PgConnection) -> Result<Vec<Set>, Error> {
        use diesel::RunQueryDsl;

        diesel::select(self::sql::tag_list(
            self.kind,
            &self.include,
            &self.exclude,
            self.limit,
            self.offset,
        ))
        .get_result(conn)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) mod sql {
    use diesel::sql_types::{Array, Int8};

    use crate::db::sql::{Bucket, Set};

    sql_function!(fn tag_list(kind: Bucket, include: Array<Set>, exclude: Array<Set>, limit: Int8, offset: Int8) -> Array<Set>);
}
