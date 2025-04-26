use diesel::prelude::*;
use diesel::{Queryable, Insertable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub done: bool,
    pub username: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = tasks)]
pub struct NewTask<'a> {
    pub title: &'a str,
    pub done: bool,
    #[serde(skip_deserializing)]
    pub username: &'a str,
}

#[derive(Queryable, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}