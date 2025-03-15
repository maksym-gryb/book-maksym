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


#[derive(Serialize, Deserialize, JsonSchema)]
struct Item {
    id: i32,
    name: String,
    price: f64,
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
fn rocket() -> _ {

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
    .mount("/", openapi_get_routes![get_items])
    .attach(cors)
    .mount("/swagger", make_swagger_ui(&get_docs()))
}
