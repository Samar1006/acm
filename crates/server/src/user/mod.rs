use axum::{
    routing::{get, post},
    Router,
};

mod edit;
mod me;
mod progress;
mod star_count;
mod submissions;
mod user_info;

pub fn routes() -> Router {
    Router::new()
        .route(
            "/username/:username/submissions",
            get(submissions::submissions),
        )
        .route("/username/:username/progress", get(progress::progress))
        .route("/username/:username", get(user_info::username))
        .route("/id/:id", get(user_info::id))
        .route("/star-count/:id", get(star_count::star_count))
        .route("/edit/:user_id", post(edit::edit))
        .route("/me", get(me::me))
}

/// Thin wrappers so tests in other modules can call these handlers.
#[cfg(test)]
pub(crate) async fn progress_for_tests(
    pool: &sqlx::SqlitePool,
    username: &str,
) -> Result<axum::Json<serde_json::Value>, crate::error::ServerError> {
    let axum::Json(progress) = progress::progress(
        axum::extract::Path(username.to_string()),
        axum::Extension(pool.clone()),
    )
    .await?;
    Ok(axum::Json(serde_json::to_value(progress).unwrap()))
}

#[cfg(test)]
pub(crate) async fn submissions_for_tests(
    pool: &sqlx::SqlitePool,
    username: &str,
) -> Result<axum::Json<serde_json::Value>, crate::error::ServerError> {
    let query = axum::extract::Query(serde_json::from_value(serde_json::json!({})).unwrap());
    let axum::Json(items) = submissions::submissions(
        axum::extract::Path(username.to_string()),
        query,
        axum::Extension(pool.clone()),
    )
    .await?;
    Ok(axum::Json(serde_json::to_value(items).unwrap()))
}
