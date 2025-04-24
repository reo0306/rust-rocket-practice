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

#[put("/tasks/<task_id>", format = "json", data = "<task_update>")]
fn update_task(<task_id: i32, task_update: Json<models::NewTask>, cookies: &CookieJar<'_>) -> Result<&'static str, Status> {
    let current_username = match cookies.get("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(Status::Unauthorized)
    }

    use crate::schema::tasks::dsl::*;
    let mut connection = establish_connection();

    let task_count: i64 = tasks
        .filter(id.eq(task_id))
        .filter(username.eq(&current_username))
        .count()
        .get_result(&mut connection)
        .expect("Error counting tasks");

    if task_count == 0 {
        return Err(Status::Forbidden);
    }

    let mut task_data = task_update.into_inner();
    task_data.username = &current_username;

    diesel::update(tasks.find(task_id))
        .set((
            title.eq(task_data.title),
            done.eq(task_data.done)
        ))
        .execute(&mut connection)
        .expect("Error updating task");

    Ok("Task updated")
}

#[delete("/tasks/<task_id>")]
fn delete_task(task_id: i32, cookies: &CookieJar<'_>) -> Result<&'static str, Status> {
    let current_username = match cookies.get("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(Status::Unauthorized)
    }

    use crate::schema::tasks::dsl::*;
    let mut connection = establish_connection();

    let task_count: i64 = tasks
        .filter(id.eq(task_id))
        .filter(username.eq(&current_username))
        .count()
        .get_result(&mut connection)
        .expect("Error counting tasks");

    if task_count == 0 {
        return Err(Status::Forbidden);
    }

    diesel::delete(tasks.find(task_id))
        .execute(&mut connection)
        .expect("Error deleting task");

    Ok("Task deleted")
}

#[post("/register", format = "json", data = "<user>")]
fn register(user: Json<models::NewUser>) -> &'static str {
    use crate::schema::tasks::dsl::*;
    let mut connection = establish_connection();

    let hashed_pw = hash::<&str>(user.password, DEFAULT_COST).expect("Failed to hash password");
    let new_user = models::NewUser {
        username: &user.username,
        password: &hashed_pw,
    }

    match diesel::insert_into(users).values(&new_user).execute(&mut connection) {
        Ok(_) => "User registered successfully",
        Err(_) => "Failed to register user",
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
