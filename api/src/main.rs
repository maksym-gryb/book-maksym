#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};

use rocket::get;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::okapi::schemars::JsonSchema;

use rocket::form::FromForm;
use rocket_okapi::okapi::schemars;
use rocket_okapi::settings::UrlObject;

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool, FromRow, Row};



#[derive(Serialize, Deserialize, JsonSchema)]
struct Item {
    id: i32,
    name: String,
    price: f64,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, FromRow, Debug)]
struct Event {
    id: i32,
    title: String,
    start_date: String,
    end_date: String
}

const DB_URL: &str = "sqlite://sqlite.db";

#[openapi(tag = "Events")]
#[get("/events")]
async fn get_events() -> Json<Vec<Event>> {
    let db = SqlitePool::connect(DB_URL).await.unwrap();

    let users = sqlx::query_as::<_, Event>(
        "SELECT * FROM events"
    ).fetch_all(&db)
    .await
    .unwrap();

    return Json(users);
}

#[openapi(tag = "Items")]
#[get("/items")]
fn get_items() -> Json<Vec<Item>> {
    Json(vec![
        Item { id: 1, name: "Item 1".to_string(), price: 100.0 },
        Item { id: 2, name: "Item 2".to_string(), price: 200.0 },
    ])
}

fn get_docs() -> SwaggerUIConfig {
    use rocket_okapi::settings::UrlObject;

    SwaggerUIConfig {
    url: "../openapi.json".to_string(),
    ..Default::default()
    }
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

    let result = sqlx::query("
        CREATE TABLE IF NOT EXISTS events
        (id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, start_date TEXT NULL, end_date TEXT NULL);"
    ).execute(&db).await.unwrap();

    println!("Create events table result: {:?}", result);

    let truncateResult = sqlx::query("
        DELETE FROM events
    ").execute(&db).await.unwrap();

    let insertResult = sqlx::query("
        INSERT INTO events
        (title, start_date, end_date)
        VALUES
        ('celebration', date('now'), date('now', '+3 day')),
        ('sacrifice', date('now', '-2 day'), date('now', '+5 day'))
    ").execute(&db).await.unwrap();

    println!("Inserts result: {:?}", insertResult);


    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:5173"]);

    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error creating CORS fairing");


    rocket::build()
    .mount("/", openapi_get_routes![get_items, get_events])
    .attach(cors)
    .mount("/swagger", make_swagger_ui(&get_docs()))
}

// #[get("/items")]
// async fn main() {
// }