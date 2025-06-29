use crate::{
    out,
    session::session,
    state::{Senv, Spool},
    user::User,
};
use axum::{
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, header::SET_COOKIE},
    response::Html,
};
use axum_cookie::CookieManager;
use minijinja::context;

pub async fn index(jar: CookieManager, State(env): Senv, State(pool): Spool) -> Html<String> {
    let (user, _jar) = session(State(pool), jar).await;

    // let css = turf::style_sheet_values!("./static/css/style.scss").0;

    let out = out(&env, "index.html", context! { user }).unwrap();

    Html(out)
}

pub async fn today() -> Html<&'static str> {
    Html(include_str!("../templates/fragment-todaylist.html"))
}

pub async fn tomorrow() -> Html<&'static str> {
    Html(include_str!("../templates/fragment-tomorrowlist.html"))
}
