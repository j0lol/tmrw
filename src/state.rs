use axum::extract::{FromRef, State};
use deadpool_sqlite::Pool;
use minijinja::Environment;

pub type Senv = State<Environment<'static>>;
pub type Spool = State<Pool>;

#[derive(Clone)]
pub struct AppState {
    pub env: Environment<'static>,
    pub pool: Pool,
}

impl FromRef<AppState> for Environment<'static> {
    fn from_ref(app_state: &AppState) -> Environment<'static> {
        app_state.env.clone()
    }
}

impl FromRef<AppState> for Pool {
    fn from_ref(app_state: &AppState) -> Pool {
        app_state.pool.clone()
    }
}
