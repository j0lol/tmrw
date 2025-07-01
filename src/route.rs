use crate::{
    out,
    session::session,
    state::{Senv, Spool},
};
use axum::{extract::State, response::Html};
use axum_cookie::CookieManager;
use minijinja::context;

pub async fn index(jar: CookieManager, State(env): Senv, State(pool): Spool) -> Html<String> {
    let (user, _jar) = session(State(pool), jar).await;

    // let css = turf::style_sheet_values!("./static/css/style.scss").0;

    let out = out(&env, "index.html", context! { user }).unwrap();

    Html(out)
}
