table! {
    use diesel::sql_types::*;
    use crate::db::sql::*;

    set_tag (id) {
        id -> Uuid,
        tag -> Set,
        set -> Set,
        created_at -> Timestamptz,
    }
}
