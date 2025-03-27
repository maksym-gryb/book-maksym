use rocket::{get, State};
use rocket_dyn_templates::{context, Template};
use rocket::tokio::time::{sleep, Duration};

use crate::data::Event;
use crate::state::AppState;

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {
        title: "Home Page",
        name: "API for the best website ever!"
    })
}

#[get("/events")]
fn events() -> Template {
    Template::render("events", context! {
        title: "Events"
    })
}

#[get("/events/load")]
async fn events_load(state: &State<AppState>) -> Template {
    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events"
    ).fetch_all(&state.db)
    .await
    .unwrap();

    // SIMULATE: slow internet
    sleep(Duration::from_millis(1000)).await;
    // END SIMULATE

    Template::render("components/event", context! {
        title: "Events",
        events: events
    })
}

#[get("/login")]
fn login_page() -> Template {
    Template::render("login", context! {
        title: "Login"
    })
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, events, events_load, login_page]
}
