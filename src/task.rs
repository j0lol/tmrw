use std::collections::HashMap;

use axum::{
    Form, Json,
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
};
use axum_cookie::CookieManager;
use serde::{Deserialize, Serialize};

use crate::{SqlRes, session::session, state::Spool};

#[derive(Serialize)]
pub struct Task {
    id: u64,
    text: String,
    #[serde(skip_serializing)]
    date: chrono::NaiveDate,
    checked: bool,
}

impl Task {
    fn html(&self) -> String {
        let text = ammonia::clean_text(&self.text);

        format!(
            r#"
            <task-item {}message="{}" task_id="{}"></task-item>
            "#,
            if self.checked { "checked " } else { "" },
            text,
            self.id
        )
    }
}

#[derive(Deserialize)]
pub enum TaskWhen {
    #[serde(rename = "today")]
    Today,

    #[serde(rename = "tomorrow")]
    Tomorrow,
}

#[derive(Deserialize)]
pub struct NewTask {
    text: String,

    #[serde(rename = "when")]
    when: TaskWhen,
}
pub async fn new_task(
    State(pool): Spool,
    jar: CookieManager,
    form: Form<NewTask>,
) -> impl IntoResponse {
    let (user, _jar) = session(State(pool.clone()), jar).await;

    let user = user.expect("Unauthenticated!");

    let text: String = form.text.clone();
    let text: String = text.trim().to_string();

    let date = match form.when {
        TaskWhen::Today => user.today(),
        TaskWhen::Tomorrow => user.tomorrow(),
    };

    let conn = pool.get().await.unwrap();
    let task = conn
        .interact(move |conn| {
            let task = conn.query_row(
                "INSERT INTO tasks (text, date, user) VALUES (?1, ?2, ?3) RETURNING rowid, CAST(text as TEXT), date, checked",
                (text, date, user.id),
                |row| {
                    Ok(Task {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        date: row.get(2)?,
                        checked: row.get(3)?,
                    })
                },
            )?;

            SqlRes::Ok(task)
        })
        .await
        .unwrap()
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("HX-Trigger", HeaderValue::from_static("taskCreated"));
    (headers, Json(serde_json::to_string(&task).unwrap()))
}

pub async fn delete_task(
    State(pool): Spool,
    form: Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = form.get("id").unwrap().clone();

    let conn = pool.get().await.unwrap();
    conn.interact(move |conn| {
        conn.execute("DELETE FROM tasks WHERE rowid = ?1", [id])?;

        SqlRes::Ok(())
    })
    .await
    .unwrap()
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("HX-Trigger", HeaderValue::from_static("taskDeleted"));
    headers
}

pub async fn check_task(
    State(pool): Spool,
    form: Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = form.get("id").unwrap().clone();

    let conn = pool.get().await.unwrap();
    let task: Task = conn
        .interact(move |conn| {
            let mut task = conn.query_row(
                "SELECT rowid, CAST(text as TEXT), date, checked FROM tasks WHERE rowid = ?1",
                [id.clone()],
                |row| {
                    Ok(Task {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        date: row.get(2)?,
                        checked: row.get(3)?,
                    })
                },
            )?;

            task.checked = !task.checked;

            conn.execute(
                "UPDATE tasks SET checked = ?1 WHERE rowid = ?2",
                (task.checked, id),
            )?;

            SqlRes::Ok(task)
        })
        .await
        .unwrap()
        .unwrap();

    let headers = HeaderMap::new();
    // headers.insert("HX-Trigger", HeaderValue::from_static("taskUpdated"));
    (headers)
}

pub async fn pushback_task(
    State(pool): Spool,
    jar: CookieManager,
    form: Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let (user, _jar) = session(State(pool.clone()), jar).await;

    let user = user.expect("Unauthenticated!");

    let id = form.get("id").unwrap().clone();

    let conn = pool.get().await.unwrap();
    conn.interact(move |conn| {
        let mut task = conn.query_row(
            "SELECT rowid, CAST(text as TEXT), date, checked FROM tasks WHERE rowid = ?1",
            [id.clone()],
            |row| {
                Ok(Task {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    date: row.get(2)?,
                    checked: row.get(3)?,
                })
            },
        )?;

        task.date = user.tomorrow();

        conn.execute(
            "UPDATE tasks SET date = ?1 WHERE rowid = ?2",
            (task.date, id),
        )?;

        SqlRes::Ok(())
    })
    .await
    .unwrap()
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("HX-Trigger", HeaderValue::from_static("taskUpdated"));
    headers
}

#[derive(Deserialize)]
pub struct ListTasks {
    #[serde(rename = "when")]
    pub when: TaskWhen,
}

pub struct Unauthenticated;

pub async fn tasks_internal(
    State(pool): Spool,
    jar: CookieManager,
    form: ListTasks,
) -> Result<Vec<Task>, Unauthenticated> {
    let (user, _jar) = session(State(pool.clone()), jar).await;

    let Some(user) = user else {
        return Err(Unauthenticated);
    };

    let date = match form.when {
        TaskWhen::Today => user.today(),
        TaskWhen::Tomorrow => user.tomorrow(),
    };

    let user_id = user.id;
    let conn = pool.get().await.unwrap();
    let tasks = conn
        .interact(move |conn| {
            let mut stmt = conn.prepare(
                    match form.when {
                        TaskWhen::Today => {
                            "SELECT rowid, CAST(text as TEXT), date, checked FROM tasks WHERE date <= ?1 AND user = ?2"

                        },
                        TaskWhen::Tomorrow => {
                            "SELECT rowid, CAST(text as TEXT), date, checked FROM tasks WHERE date = ?1 AND user = ?2"

                        }
                    }
                // "SELECT rowid, text, date, checked FROM tasks WHERE date = ?1 AND user = ?2",
            )?;

            let tasks: SqlRes<Vec<Task>> = stmt
                .query_map((date, user_id), |row| {
                    Ok(Task {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        date: row.get(2)?,
                        checked: row.get(3)?,
                    })
                })?
                .collect();

            SqlRes::Ok(tasks.unwrap())
        })
        .await
        .unwrap()
        .unwrap();

    Ok(tasks)
}

pub async fn tasks(
    State(pool): Spool,
    jar: CookieManager,
    Form(form): Form<ListTasks>,
) -> Html<String> {
    let (user, _jar) = session(State(pool.clone()), jar).await;

    let user = user.expect("Unauthenticated!");

    let date = match form.when {
        TaskWhen::Today => user.today(),
        TaskWhen::Tomorrow => user.tomorrow(),
    };

    let user_id = user.id;
    let conn = pool.get().await.unwrap();
    let tasks = conn
        .interact(move |conn| {
            let mut stmt = conn.prepare(
                    match form.when {
                        TaskWhen::Today => {
                            "SELECT rowid, CAST(text as TEXT), date, checked FROM tasks WHERE date <= ?1 AND user = ?2"

                        },
                        TaskWhen::Tomorrow => {
                            "SELECT rowid, CAST(text as TEXT), date, checked FROM tasks WHERE date = ?1 AND user = ?2"

                        }
                    }
                // "SELECT rowid, text, date, checked FROM tasks WHERE date = ?1 AND user = ?2",
            )?;

            let tasks: SqlRes<String> = stmt
                .query_map((date, user_id), |row| {
                    Ok(Task {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        date: row.get(2)?,
                        checked: row.get(3)?,
                    }
                    .html())
                })?
                .collect();

            SqlRes::Ok(tasks.unwrap())
        })
        .await
        .unwrap()
        .unwrap();

    Html(tasks)
}
