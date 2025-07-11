use crate::{
    route::index,
    state::AppState,
    task::{check_task, delete_task, new_task, pushback_task, tasks},
    user::{login, new_user, user_info},
};
use axum::{
    Router,
    routing::{get, post},
};
use axum_cookie::CookieLayer;
use deadpool_sqlite::{Config, Runtime};
use minijinja::{Environment, path_loader};
use serde::Serialize;
use std::error::Error;
use tower_http::services::ServeDir;

mod route;
pub mod session;
pub mod state;
mod task;
pub mod user;

type Res<T> = Result<T, Box<(dyn Error)>>;
type SqlRes<T> = deadpool_sqlite::rusqlite::Result<T>;

#[tokio::main]
async fn main() -> Res<()> {
    let port: u32 = std::env::var("PORT")
        .unwrap_or("3000".to_string())
        .parse()
        .unwrap();

    let env = jinja_env();

    let cfg = Config::new("db.sqlite3");
    let pool = cfg.create_pool(Runtime::Tokio1).unwrap();

    let conn = pool.get().await.unwrap();
    conn.interact(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                text STRING NOT NULL,
                date STRING NOT NULL,
                user INTEGER NOT NULL,
                checked BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                password STRING NOT NULL UNIQUE,
                timezone STRING NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id STRING NOT NULL UNIQUE,
                user INTEGER
            )",
            [],
        )?;

        SqlRes::Ok(())
    })
    .await??;

    // build our application with a route
    let app = Router::new()
        .route("/", get(index))
        .route("/task/new", post(new_task))
        .route("/task/delete", post(delete_task))
        .route("/task/list", post(tasks))
        .route("/task/complete", post(check_task))
        .route("/task/pushback", post(pushback_task))
        .route("/user/new", post(new_user))
        .route("/user/login", post(login))
        .route("/user/info", post(user_info))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(AppState { env, pool })
        .layer(CookieLayer::default());

    // run it
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    println!(
        "listening on {}. Change this with the PORT env var.",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;

    Ok(())
}

fn jinja_env() -> Environment<'static> {
    let mut env = Environment::new();

    env.set_loader(path_loader("templates"));
    env
}

fn out(
    env: &Environment<'static>,
    file_name: &str,
    context: impl Serialize,
) -> Result<String, Box<dyn Error>> {
    let template = env.get_template(file_name)?;
    let str = template.render(context)?;

    Ok(str)
}
