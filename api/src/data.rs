use rocket::serde::{Deserialize, Serialize};
use sqlx::FromRow;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::okapi::schemars;

#[derive(Serialize, Deserialize, JsonSchema, Clone, FromRow, Debug, FromForm)]
pub struct Event {
    pub id: Option<i64>,
    pub title: String,
    pub start_date: String,// TODO: find a way to convert to Date/DateTime
    pub end_date: String
}