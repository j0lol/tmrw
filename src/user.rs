use axum::{
    Form, Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{Html, Redirect},
};
use axum_cookie::CookieManager;
use chrono::{DateTime, Days, NaiveDate, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    SqlRes,
    session::{session, session_login},
    state::Spool,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u64,
    pub pass: String,
    pub timezone: chrono_tz::Tz,
}

impl User {
    /// "Now", from the perspective of the user.
    pub fn now(&self) -> DateTime<Tz> {
        use chrono::TimeZone;
        let date = Utc::now().naive_utc();
        self.timezone.from_utc_datetime(&date)
    }

    /// Today, from the perspective of the user.
    pub fn today(&self) -> NaiveDate {
        self.now().date_naive()
    }

    /// Tomorrow, from the perspective of the user.
    pub fn tomorrow(&self) -> NaiveDate {
        self.now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap()
    }
}

#[derive(Deserialize)]
pub struct Login {
    pass: String,
}

pub async fn login(
    State(pool): Spool,
    jar: CookieManager,
    Form(payload): Form<Login>,
) -> (HeaderMap, Redirect) {
    let (_, mut jar) = session(State(pool.clone()), jar).await;

    let conn = pool.get().await.unwrap();
    let userid = conn
        .interact(move |conn| {
            let mut stmt = conn.prepare("SELECT (rowid) FROM users  WHERE password = ?1")?;
            let userid: u32 = stmt.query_row([payload.pass.trim()], |row| row.get(0))?;

            SqlRes::Ok(userid)
        })
        .await
        .unwrap()
        .unwrap();

    session_login(pool, userid, &mut jar).await;

    let mut map = HeaderMap::new();
    map.insert("HX-Trigger", HeaderValue::from_static("login"));

    (map, Redirect::to("/"))
}

#[derive(Deserialize)]
pub struct NewUser {
    timezone: String,
}

pub async fn new_user(
    State(pool): Spool,
    jar: CookieManager,
    Json(payload): Json<NewUser>,
) -> (HeaderMap, StatusCode) {
    let (_, mut jar) = session(State(pool.clone()), jar).await;

    let pass = generate_password();
    let timezone: Tz = payload.timezone.parse().unwrap();

    let conn = pool.get().await.unwrap();
    let pass_clone = pass.clone();
    let rowid = conn
        .interact(move |conn| {
            let mut stmt = conn.prepare(
                "INSERT INTO users (password, timezone) VALUES (?1, ?2) RETURNING rowid",
            )?;
            let rowid: u32 =
                stmt.query_row([pass_clone, timezone.to_string()], |row| row.get(0))?;

            SqlRes::Ok(rowid)
        })
        .await
        .unwrap()
        .unwrap();

    session_login(pool, rowid, &mut jar).await;

    let mut map = HeaderMap::new();
    map.insert("HX-Trigger", HeaderValue::from_static("login"));

    (map, StatusCode::OK)
}

fn generate_password() -> String {
    let get_word = || random_word::get(random_word::Lang::En);

    [get_word(), get_word(), get_word(), get_word()].join(" ")
}

type MaybeJson<T> = Result<Json<T>, ()>;
fn yes_json<T>(i: T) -> MaybeJson<T> {
    Ok(axum::Json(i))
}
fn no_json<T>() -> MaybeJson<T> {
    Err(())
}

pub async fn user_info(State(pool): Spool, jar: CookieManager) -> (StatusCode, MaybeJson<User>) {
    let (user, _jar) = session(State(pool), jar).await;

    let Some(user) = user else {
        return (StatusCode::UNAUTHORIZED, no_json());
    };

    (StatusCode::OK, yes_json(user))
}
