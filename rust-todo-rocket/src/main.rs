#[macro_use] extern crate rocket;

use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::http::{Cookie, CookieJar, Status};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use bcrypt::{hash, verify, DEFAULT_COST};

mod schema;
mod models;

#[get("/")]
async fn serve_index() -> Option<NamedFile> {
    NamedFile::open("static/index.html").await.ok()
}

#[get("/tasks")]
fn get_tasks(cookies: &CookieJar<'_>) -> Result<Json<Vec<models::Task>>, Status> {
    let current_username = match cookies.get("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(Status::Unauthorized)
    }

    use crate::schema::tasks::dsl::*;
    let mut connection = establish_connection();

    let results = tasks
        .filter(username.eq(&current_username))
        .load::<models::Task>(&mut connection)
        .expect("Error loading tasks");

    Ok(Json(results))
}

#[post("/tasks", format = "json", data = "<new_task>")]
fn create_task(new_task: Json<models::NewTask>, cookies: &CookieJar<'_>) -> Result<Json<models::Task>, Status> {
    let current_username = match cookies.get("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(Status::Unauthorized)
    }

    use crate::schema::tasks::dsl::*;
    let mut connection = establish_connection();

    let mut new_task_data = new_task.into_inner();
    new_task_data.username = &current_username;

    diesel::insert_into(tasks)
        .values(&new_task_data)
        .execute(&mut connection)
        .expect("Error inserting new task");

    let inserted_task = tasks
        .filter(username.eq(&current_username))
        .order(id.desc())
        .first::<models::Task>(&mut connection)
        .unwrap();

    Ok(Json(inserted_task))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
