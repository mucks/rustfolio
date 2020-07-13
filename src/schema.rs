table! {
    users (id) {
        id -> Varchar,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        created_on -> Timestamp,
        last_login -> Nullable<Timestamp>,
    }
}
