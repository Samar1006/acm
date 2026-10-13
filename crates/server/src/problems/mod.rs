use axum::{
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

mod edit;
pub(crate) mod extras;
mod history;
mod index;
mod leaderboard;
mod new;
mod problem;
mod recent_submission;
mod recent_tests;
mod tests;

#[derive(Serialize, Deserialize, Clone, Type)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Serialize, Clone, FromRow)]
pub struct Problem {
    pub id: i64,

    // A competition id, if the problem is in one.
    pub competition_id: Option<i64>,

    /// Problem title
    pub title: String,

    /// Problem description (markdown formatted)
    pub description: String,

    /// Code that parses standard input and outputs to standard out
    pub runner: String,

    /// Template that's shown when you start a problem
    pub template: String,

    pub runtime_multiplier: Option<f64>,

    pub visible: bool,

    pub difficulty: Difficulty,
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index::problems))
        .route("/new", post(new::new))
        .route("/tags", get(extras::tags))
        .route("/:problem_id", get(problem::problem))
        .route("/:problem_id/edit", post(edit::edit))
        .route("/:problem_id/rust-template", get(problem::rust_template))
        .route(
            "/:problem_id/python-template",
            get(problem::python_template),
        )
        .route("/:problem_id/examples", get(extras::examples))
        .route("/:problem_id/meta", get(extras::meta))
        .route("/:problem_id/bookmark", post(extras::bookmark))
        .route(
            "/:problem_id/draft",
            get(extras::draft).post(extras::save_draft),
        )
        .route("/:problem_id/tests", get(tests::tests))
        .route("/:problem_id/tests/:test_number", get(tests::problem_test))
        .route("/:problem_id/history", get(history::history))
        .route(
            "/:problem_id/leaderboard/users",
            get(leaderboard::leaderboard_users),
        )
        .route(
            "/:problem_id/leaderboard/submissions",
            get(leaderboard::leaderboard_submissions),
        )
        .route(
            "/:problem_id/leaderboard/slow-submissions",
            get(leaderboard::leaderboard_slow_submissions),
        )
        .route(
            "/:problem_id/recent-submission",
            get(recent_submission::recent_submission),
        )
        .route("/:problem_id/recent-tests", get(recent_tests::recent_tests))
        .route(
            "/:problem_id/recent-tests/:test_number",
            get(recent_tests::recent_tests_test),
        )
}

/// Thin wrappers so tests in other modules can call these handlers.
#[cfg(test)]
pub(crate) async fn edit_for_tests(
    pool: &sqlx::SqlitePool,
    claims: crate::auth::Claims,
    form: serde_json::Value,
) -> Result<(), crate::error::ServerError> {
    edit::edit(
        axum::Extension(pool.clone()),
        claims,
        axum::extract::Path(1),
        axum::Json(serde_json::from_value(form).unwrap()),
    )
    .await
}

#[cfg(test)]
pub(crate) async fn history_for_tests(
    pool: &sqlx::SqlitePool,
    problem_id: i64,
    claims: crate::auth::Claims,
) -> Result<axum::Json<serde_json::Value>, crate::error::ServerError> {
    let query = axum::extract::Query(serde_json::from_value(serde_json::json!({})).unwrap());
    let axum::Json(items) = history::history(
        axum::extract::Path(problem_id),
        axum::Extension(pool.clone()),
        query,
        claims,
    )
    .await?;
    Ok(axum::Json(serde_json::to_value(items).unwrap()))
}

#[cfg(test)]
pub(crate) async fn list_for_tests(
    pool: &sqlx::SqlitePool,
    query: &str,
    claims: crate::auth::Claims,
) -> Result<axum::Json<serde_json::Value>, crate::error::ServerError> {
    let uri: axum::http::Uri = format!("/?{query}").parse().unwrap();
    let axum::Json(items) = index::problems(
        axum::Extension(pool.clone()),
        axum::extract::Query::try_from_uri(&uri).unwrap(),
        axum::extract::Query::try_from_uri(&uri).unwrap(),
        claims,
    )
    .await?;
    Ok(axum::Json(serde_json::to_value(items).unwrap()))
}

#[cfg(test)]
pub(crate) async fn new_for_tests(
    pool: &sqlx::SqlitePool,
    claims: crate::auth::Claims,
    form: &str,
) -> Result<i64, crate::error::ServerError> {
    let (broadcast, _) = tokio::sync::broadcast::channel(8);
    let axum::Json(created) = new::new(
        axum::Extension(pool.clone()),
        axum::Extension(broadcast),
        claims,
        axum::Json(serde_json::from_str(form).unwrap()),
    )
    .await?;
    Ok(serde_json::to_value(created).unwrap()["id"]
        .as_i64()
        .unwrap())
}
