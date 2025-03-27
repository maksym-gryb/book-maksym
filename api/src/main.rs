#[macro_use] extern crate rocket;

mod cors;
mod htmx;
mod auth;
use auth::User;

mod state;
use state::AppState;

mod data;
use data::Event;

use rocket::serde::{json::Json, Deserialize, Serialize};

use rocket::{get, post, State};
use rocket::response::Redirect;
use rocket::form::Form;
use rocket::http::Status;
use rocket_okapi::{openapi, openapi_get_routes};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;

use rocket_dyn_templates::Template;

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool, Row};


#[derive(Serialize, Deserialize, JsonSchema)]
struct ErrorResponse {
    error: String,
    message: String,
}

const DB_URL: &str = "sqlite://sqlite.db";

// #[openapi(tag = "Events")]
#[get("/events")]
async fn get_events(state: &State<AppState>, _user: User) -> Json<Vec<Event>> {
    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events"
    ).fetch_all(&state.db)
    .await
    .unwrap();

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
        (id TEXT PRIMARY KEY NOT NULL, user_id INT NOT NULL, created_on TEXT NOT NULL)"// foreign key for user_id?
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
    .mount("/", auth::routes())
    .mount("/htmx", htmx::routes())
    .mount("/", openapi_get_routes![/*get_events, */create_event])
    .attach(cors::CORS)
    .attach(Template::fairing())
    .mount("/swagger", make_swagger_ui(&get_docs()))
    .manage(AppState {db : db})
    // .attach(Template::custom(|engines| {
    //     htmx::customize(&mut engines.tera);
    // }))
}