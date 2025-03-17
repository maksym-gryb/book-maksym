#[macro_use] extern crate rocket;

mod cors;

use rocket::serde::{json::Json, Deserialize, Serialize};

use rocket::{get, post};
use rocket::form::{Form, FromForm};
use rocket::State;
use rocket_okapi::{openapi, openapi_get_routes};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;

// use rocket_auth::{Users, Error, Auth, Signup, Login};

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool, FromRow, Row};

#[derive(Serialize, Deserialize, JsonSchema, Clone, FromRow, Debug, FromForm)]
struct Event {
    id: Option<i32>,
    title: String,
    start_date: String,// TODO: find a way to convert to Date/DateTime
    end_date: String
}

struct AppState{
    db : SqlitePool
}

const DB_URL: &str = "sqlite://sqlite.db";

#[openapi(tag = "Events")]
#[get("/events")]
async fn get_events(state : &State<AppState>) -> Json<Vec<Event>> {
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
    -> Json<Event> {
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
    .await;

    let id: i32 = match result {
        Ok(r) => r.get::<i32, &str>("id"),
        Err(e) => panic!("could not save row: {}", e)
    };

    Json(Event{
        id: Some(id),
        title: event.title.to_string(),
        start_date: event.start_date.to_string(),
        end_date: event.end_date.to_string()
    })
}

fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
    url: "../openapi.json".to_string(),
    ..Default::default()
    }
}


// #[post("/signup", data="<form>")]
// async fn signup(form: Form<Signup>, auth: Auth<'_>) {
//     auth.signup(&form).await;
//     auth.login(&form.into());
// }

// #[post("/login", data="<form>")]
// fn login(form: Form<Login>, auth: Auth) {
//     auth.login(&form);
// }

// #[get("/logout")]
// fn logout(auth: Auth) {
//     auth.logout();
// }

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

    // let cors = CorsOptions {
    //     allowed_origins: AllowedOrigins::some_exact(&["http://localhost:5173"]),
    //     allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
    //     allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
    //     allow_credentials: true,
    //     ..Default::default()
    // }
    // .to_cors()
    // .expect("error creating CORS fairing");

    rocket::build()
    .mount("/", openapi_get_routes![get_events, create_event])
    // .mount("/", routes![signup, login, logout])
    .attach(cors::CORS)
    .mount("/swagger", make_swagger_ui(&get_docs()))
    .manage(AppState {db : db})
}