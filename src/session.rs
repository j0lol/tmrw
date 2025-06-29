use crate::{SqlRes, state::Spool, user::User};
use axum::extract::State;
use deadpool_sqlite::{Pool, rusqlite::types::Null};

use axum_cookie::{CookieManager as CookieJar, cookie::Cookie};

pub async fn session_login(pool: Pool, userid: u32, jar: &mut CookieJar) {
    let sessid = jar.get("id").unwrap().value().to_owned();

    dbg!(&sessid);

    println!("running session_login?");
    let conn = pool.get().await.unwrap();
    conn.interact(move |conn| {
        let rows_updated: usize = conn
            .execute(
                "UPDATE sessions SET user = ?1 WHERE id = ?2",
                (userid, sessid),
            )
            .unwrap();

        debug_assert!(rows_updated == 1);
    })
    .await
    .unwrap();

    jar.remove("invalidate");
}

async fn session_set_cookie(id: String, jar: &mut CookieJar) {
    jar.add(Cookie::new("id", id).with_expires("Tue, 19 Jan 2038 03:14:07 GMT"));
}

async fn session_set_invalidate(jar: &mut CookieJar) {
    jar.add(Cookie::new("invalidate", "true"));
}

async fn session_load(id: String, pool: Pool, mut jar: CookieJar) -> (Option<User>, CookieJar) {
    let conn = pool.get().await.unwrap();
    let user = conn
        .interact(|conn| {
            let userid =
                conn.query_row("SELECT (user) FROM sessions WHERE id == ?1", [id], |row| {
                    row.get::<usize, Option<u32>>(0)
                })?;

            if let Some(userid) = userid {
                let user = conn.query_row(
                    "SELECT rowid, password, timezone FROM users WHERE rowid == ?1",
                    [userid],
                    |row| {
                        Ok(User {
                            id: row.get(0)?,
                            pass: row.get(1)?,
                            timezone: row.get::<usize, String>(2)?.parse().unwrap(),
                        })
                    },
                )?;
                SqlRes::Ok(Some(user))
            } else {
                SqlRes::Ok(None)
            }
        })
        .await
        .unwrap()
        .unwrap();

    if let Some(user) = user {
        (Some(user), jar)
    } else {
        session_set_invalidate(&mut jar).await;
        (None, jar)
    }
}

async fn session_create(pool: Pool, mut jar: CookieJar) -> (Option<User>, CookieJar) {
    let id = session_id_new();

    let id_clone = id.clone();
    let conn = pool.get().await.unwrap();
    conn.interact(|conn| {
        conn.execute(
            "INSERT INTO sessions (id, user) VALUES (?1, ?2)",
            (id_clone, Null),
        )
    })
    .await
    .unwrap()
    .unwrap();

    session_set_cookie(id, &mut jar).await;
    session_set_invalidate(&mut jar).await;

    (None, jar)
}

pub async fn session(State(pool): Spool, jar: CookieJar) -> (Option<User>, CookieJar) {
    if let Some(id) = jar.get("id") {
        session_load(id.value().to_owned(), pool, jar).await
    } else {
        session_create(pool, jar).await
    }
}

fn session_id_new() -> String {
    rand::distr::SampleString::sample_string(&rand::distr::Alphanumeric, &mut rand::rng(), 16)
}
