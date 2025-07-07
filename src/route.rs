use crate::{
    out,
    session::session,
    state::{Senv, Spool},
    task::{ListTasks, TaskWhen, tasks, tasks_internal},
};
use axum::{Form, extract::State, response::Html};
use axum_cookie::CookieManager;
use minijinja::context;

pub async fn index(jar: CookieManager, State(env): Senv, State(pool): Spool) -> Html<String> {
    let (user, _jar) = session(State(pool.clone()), jar.clone()).await;

    let today = tasks_internal(
        State(pool.clone()),
        jar.clone(),
        ListTasks {
            when: TaskWhen::Today,
        },
    )
    .await;
    let tomorrow = tasks_internal(
        State(pool),
        jar,
        ListTasks {
            when: TaskWhen::Tomorrow,
        },
    )
    .await;

    let out = out(&env, "index.html", context! { user, today, tomorrow }).unwrap();

    Html(out)
}
