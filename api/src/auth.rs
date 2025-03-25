use uuid::Uuid;
use rocket::outcome::IntoOutcome;

use rocket::http::{CookieJar, Status};
use rocket::request::{self, FromRequest};
use rocket::{get, post, State, Response, Request};
use rocket::form::{Form, FromForm};
use rocket::serde::{json::Json, Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool, FromRow, Row};

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use crate::state::AppState;

#[derive(Serialize, Deserialize, Clone, Debug, FromForm, FromRow)]
pub struct User{
    pub id: Option<i64>,
    pub username: String,
    pub password: Option<String>,
    pub role: String
}

#[derive(FromRow)]
pub struct Session{
    id: String,
    user_id: i64
}

pub struct SessionId(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    // type Error = std::convert::Infallible;
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, ()/*Self::Error*/> {
        let session_id: SessionId = rocket::outcome::try_outcome!(
            request.cookies()
            .get("session_id")// TODO: make private // TODO: make const string
            .and_then(|cookie| cookie.value().parse().ok())
            .map(SessionId)
            .or_forward(Status::Unauthorized));

        let state_outcome = request.guard::<&State<AppState>>().await;
        let state = rocket::outcome::try_outcome!(state_outcome);

        let session = match sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions
            ORDER BY created_on DESC
            LIMIT 1"
        ).fetch_optional(&state.db)
        .await{
            Ok(r) => match r {
                Some(s) => s,
                None => return request::Outcome::Error((Status::Unauthorized, ())),
            },
            Err(_) => return request::Outcome::Error((Status::InternalServerError, ())),
        };

        println!("[FromRequest::User] session_id := {}", &session.id);

        let user = sqlx::query_as!(
            User,
            "SELECT *
            FROM users
            WHERE id = ?
        ", session.user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();

        println!("[FromRequest::User] username := {}", &user.username);

        request::Outcome::Success(user)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, FromForm)]
pub struct Login{
    username: String,
    password: String
}

#[post("/login", data="<login>")]
async fn login(jar: &CookieJar<'_>, state: &State<AppState>, login: Form<Login>) -> Status {
    let already_logged_id: bool = match jar.get("session_id") {
        Some(s) => {
            println!("session_id := {}", s.to_string());
            true
        },
        None => false
    };

    println!("{}", &login.username);
    println!("{}", &login.password);

    let user = sqlx::query_as!(User,
        "
        SELECT id, username, password, role
        FROM users
        WHERE username = ?
    ", login.username)
    .fetch_optional(&state.db)
    .await
    .unwrap();

    match user {
        Some(u) => {

            let psw = u.password.unwrap();
            let p = psw.as_str();
            
            let parsed_hash = PasswordHash::new(p).unwrap();
            if !Argon2::default().verify_password(login.password.as_bytes(), &parsed_hash).is_ok() {
                return Status::Unauthorized;
            }

            let uuid = Uuid::new_v4().to_string();
            println!("new cookies := {}", uuid);

            sqlx::query("
                INSERT INTO sessions
                (id, user_id, created_on)
                VALUES
                ($1, $2, datetime())
            ")
            .bind(&uuid)
            .bind(&u.id)
            .execute(&state.db)
            .await
            .unwrap();

            jar.add(("session_id", uuid));
            Status::Ok
        },
        None => {
            println!("no cookies for you!");
            Status::Unauthorized
        }
    }
}

#[get("/logout")]
async fn logout(user: User, state: &State<AppState>) {
    sqlx::query!("DELETE FROM sessions WHERE user_id = ?", user.id)
    .execute(&state.db)
    .await
    .unwrap();
}

#[get("/profile")]
fn profile(user: User) -> Json<User> {
    Json(user)
}

pub fn auth_routes() -> Vec<rocket::Route> {
    routes![login, logout, profile]
}

pub async fn init_admin_user(db: &SqlitePool) {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(b"1q2w3E*", &salt).unwrap().to_string();

    let insert_admin_user = sqlx::query!("
        INSERT INTO users
        (username, password, role)
        VALUES
        ('admin', ?, 'admin')
    ", password_hash)
    .execute(db).await.unwrap();
}


// use rocket::form::Form;
// use rocket::response::{status, Redirect};
// use rocket::outcome::IntoOutcome;
// use rocket::request::{self, FlashMessage, FromRequest, Request};
// use rocket::response::{Redirect, Flash};
// use rocket::http::{CookieJar, Status};
// use uuid::Uuid;


// #[derive(FromForm)]
// struct Login<'r> {
//     username: &'r str,
//     password: &'r str
// }

// #[derive(Debug)]
// struct User(usize);

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for User {
//     type Error = std::convert::Infallible;

//     async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
//         request.cookies()
//             .get_private("user_id")
//             .and_then(|cookie| cookie.value().parse().ok())
//             .map(User)
//             .or_forward(Status::Unauthorized)
//     }
// }

// #[macro_export]
// macro_rules! session_uri {
//     ($($t:tt)*) => (rocket::uri!("/session", $crate::session:: $($t)*))
// }

// pub use session_uri as uri;

// // #[get("/")]
// // fn index(user: User) -> Template {
// //     Template::render("session", context! {
// //         user_id: user.0,
// //     })
// // }

// // #[get("/", rank = 2)]
// // fn no_auth_index() -> Redirect {
// //     Redirect::to(uri!(login_page))
// // }

// // #[get("/login")]
// // fn login(_user: User) -> Redirect {
// //     Redirect::to(uri!(index))
// // }

// // #[get("/login", rank = 2)]
// // fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
// //     Template::render("login", &flash)
// // }

// #[openapi(tag = "Login")]
// #[post("/login", data = "<login>")]
// fn post_login(jar: &CookieJar<'_>, login: Form<Login<'_>>) -> Result<Redirect, Flash<Redirect>> {
//     if login.username == "Sergio" && login.password == "password" {
//         jar.add(("session_id", Uuid::new_v4().to_string()));
//         jar.add_private(("user_id", "1"));
//         Ok(Redirect::to(uri!(index)))
//     } else {
//         Err(Flash::error(Redirect::to(uri!(login_page)), "Invalid username/password."))
//     }
// }

// #[post("/logout")]
// fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
//     jar.remove_private("user_id");
//     Flash::success(Redirect::to(uri!(login_page)), "Successfully logged out.")
// }

// pub fn auth_routes() -> Vec<rocket::Route> {
//     routes![/*index, no_auth_index,*/ login, login_page, post_login, logout]
// }