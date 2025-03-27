use uuid::Uuid;

use rocket::http::{CookieJar, Status};
use rocket::request::{self, FromRequest};
use rocket::{get, post, State, Request};
use rocket::form::{Form, FromForm};
use rocket::serde::{json::Json, Deserialize, Serialize};

use sqlx::{SqlitePool, FromRow};

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
    user_id: i64,
    #[allow(dead_code)]
    created_on: String
}

// pub struct SessionId(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    // type Error = std::convert::Infallible;
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, ()/*Self::Error*/> {
        let session_id: String = request.cookies()
            .get("session_id")// TODO: make private // TODO: make const string
            .map(|cookie| cookie.value())
            .unwrap()
            .to_string();

        let state_outcome = request.guard::<&State<AppState>>().await;
        let state = rocket::outcome::try_outcome!(state_outcome);

        let session = match sqlx::query_as!(Session,
            "SELECT * FROM sessions
            WHERE id = ?
            ORDER BY created_on DESC
            LIMIT 1", session_id
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
    println!("{}", &login.username);
    println!("{}", &login.password);

    let user: Option<User> = sqlx::query_as!(User, "
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

pub fn routes() -> Vec<rocket::Route> {
    routes![login, logout, profile]
}

pub async fn init_admin_user(db: &SqlitePool) {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(b"1q2w3E*", &salt).unwrap().to_string();

    sqlx::query("
        INSERT INTO users
        (username, password, role)
        SELECT 'admin', $1, 'admin'
        WHERE NOT EXISTS (
            SELECT 1 FROM users WHERE username = 'admin'
        )
    ")
    .bind(password_hash)
    .execute(db)
    .await
    .unwrap();
}