#[macro_use] extern crate rocket;

mod cors;
mod auth;
use auth::User;

mod state;
use state::AppState;

use sqlx::Error;

use rocket::serde::{json::Json, Deserialize, Serialize};

use rocket::{get, post, State, Response};
use rocket::response::{status, Redirect};
use rocket::form::{Form, FromForm};
use rocket::http::{CookieJar, Status};
use rocket_okapi::{openapi, openapi_get_routes};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;

use uuid::Uuid;

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool, FromRow, Row};

#[derive(Serialize, Deserialize, JsonSchema, Clone, FromRow, Debug, FromForm)]
struct Event {
    id: Option<i64>,
    title: String,
    start_date: String,// TODO: find a way to convert to Date/DateTime
    end_date: String
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ErrorResponse {
    error: String,
    message: String,
}

// #[derive(Serialize, Deserialize, JsonSchema, Clone, FromRow, Debug, FromForm)]
// struct LogimDto {
//     username: String,
//     password: String
// }

const DB_URL: &str = "sqlite://sqlite.db";

// #[openapi(tag = "Events")]
#[get("/events")]
async fn get_events(jar: &CookieJar<'_>, state: &State<AppState>, user: User) -> Json<Vec<Event>> {
    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events"
    ).fetch_all(&state.db)
    .await
    .unwrap();

    let c: String = match jar.get("session_id") {
        Some(s) => s.value().to_string(),
        None => "no cookies".to_string()
    };

    println!("session_id := {}", c);
    println!("username := {}", user.username);

    return Json(events);
}

#[openapi(tag = "Events")]
#[post("/events", data = "<event>")]
async fn create_event(state: &State<AppState>, event: Form<Event>)
    -> Result<Json<Event>, Status> {
    let result = sqlx::query(
        "INSERT INTO events
        (title, start_date, end_date)
        VALUES
        ($1, $2, $3)
        RETURNING id"
    )
    .bind(&event.title)
    .bind(&event.start_date)
    .bind(&event.end_date)
    .fetch_one(&state.db)
    .await
    .unwrap();

    let id = result.get::<i64, &str>("id");

    Ok(Json(Event{
        id: Some(id),
        title: event.title.to_string(),
        start_date: event.start_date.to_string(),
        end_date: event.end_date.to_string()
    }))
}

fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
    url: "../openapi.json".to_string(),
    ..Default::default()
    }
}

#[get("/")]
fn redirect_to_swagger() -> Redirect {
    Redirect::to(uri!("/swagger"))
}

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();

    // TODO: transfer to migrations,
    // once we know what we want
    let create_table_result = sqlx::query("
        CREATE TABLE IF NOT EXISTS events
        (id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, start_date TEXT NULL, end_date TEXT NULL);"
    ).execute(&db).await.unwrap();
    println!("Create events table result: {:?}", create_table_result);

    let create_table_result = sqlx::query("
        CREATE TABLE IF NOT EXISTS users
        (id INTEGER PRIMARY KEY NOT NULL, username TEXT NOT NULL, password TEXT NOT NULL, role TEXT NOT NULL);"
    ).execute(&db).await.unwrap();
    println!("Create users table result: {:?}", create_table_result);

    let create_table_result = sqlx::query("
        CREATE TABLE IF NOT EXISTS sessions
        (id TEXT PRIMARY KEY NOT NULL, user_id INT NOT NULL)"// foreign key for user_id?
    ).execute(&db).await.unwrap();
    println!("Create sessions table result: {:?}", create_table_result);


    auth::init_admin_user(&db).await;
    /* TEMP don't delete
    let delete_result = sqlx::query("
        DELETE FROM events
    ").execute(&db).await.unwrap();

    println!("Delete result: {:?}", delete_result);

    let insert_result = sqlx::query("
        INSERT INTO events
        (title, start_date, end_date)
        VALUES
        ('celebration', date('now'), date('now', '+3 day')),
        ('sacrifice', date('now', '-2 day'), date('now', '+5 day'))
    ").execute(&db).await.unwrap();

    println!("Inserts result: {:?}", insert_result);
    */

    rocket::build()
    .mount("/", routes![redirect_to_swagger, get_events])
    .mount("/", auth::auth_routes())
    .mount("/", openapi_get_routes![/*get_events, */create_event])
    .attach(cors::CORS)
    .mount("/swagger", make_swagger_ui(&get_docs()))
    .manage(AppState {db : db})
}