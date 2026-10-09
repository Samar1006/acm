use axum::{extract::Path, Extension, Json};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

use crate::error::{ServerError, UserError};

#[derive(Serialize, FromRow)]
pub struct Group {
    name: String,
    solved: i64,
    total: i64,
}

#[derive(Serialize, FromRow)]
pub struct Day {
    day: String,
    submissions: i64,
    accepted: i64,
}

#[derive(Serialize)]
pub struct Progress {
    solved: i64,
    total: i64,
    /// Judged submissions. Compilation errors are not counted.
    submissions: i64,
    accepted: i64,
    by_difficulty: Vec<Group>,
    by_tag: Vec<Group>,
    /// The last 30 days that had at least one judged submission.
    activity: Vec<Day>,
}

// Practice problems only: visible and outside competitions.
const SOLVED: &str = "EXISTS (SELECT 1 FROM submissions WHERE submissions.problem_id = problems.id AND submissions.user_id = ?1 AND submissions.success = true)";

pub async fn progress(
    Path(username): Path<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<Progress>, ServerError> {
    let internal = |error: sqlx::Error| {
        log::error!("{error}");
        ServerError::InternalError
    };
    let (user_id,): (i64,) = sqlx::query_as("SELECT id FROM users WHERE username = ?")
        .bind(&username)
        .fetch_one(&pool)
        .await
        .map_err(|_| UserError::NotFound(username))?;

    let by_difficulty: Vec<Group> = sqlx::query_as(&format!(
        r#"SELECT difficulty AS name, COALESCE(SUM({SOLVED}), 0) AS solved, COUNT(*) AS total
        FROM problems
        WHERE visible = true AND competition_id IS NULL
        GROUP BY difficulty
        ORDER BY CASE difficulty WHEN 'Easy' THEN 0 WHEN 'Medium' THEN 1 ELSE 2 END"#
    ))
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(internal)?;

    let by_tag: Vec<Group> = sqlx::query_as(&format!(
        r#"SELECT problem_tags.name AS name, COALESCE(SUM({SOLVED}), 0) AS solved, COUNT(*) AS total
        FROM problem_tags JOIN problems ON problems.id = problem_tags.problem_id
        WHERE problems.visible = true AND problems.competition_id IS NULL
        GROUP BY problem_tags.name
        ORDER BY problem_tags.name"#
    ))
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(internal)?;

    let judged = r#"FROM submissions
        JOIN problems ON problems.id = submissions.problem_id
        WHERE submissions.user_id = ?
        AND submissions.verdict IS NOT 'CompilationError'
        AND problems.visible = true AND problems.competition_id IS NULL"#;
    let (submissions, accepted): (i64, i64) = sqlx::query_as(&format!(
        "SELECT COUNT(*), COALESCE(SUM(submissions.success), 0) {judged}"
    ))
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(internal)?;

    let activity: Vec<Day> = sqlx::query_as(&format!(
        r#"SELECT date(submissions.time) AS day, COUNT(*) AS submissions,
            COALESCE(SUM(submissions.success), 0) AS accepted
        {judged}
        AND submissions.time >= datetime('now', '-30 days')
        GROUP BY day ORDER BY day DESC"#
    ))
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(internal)?;

    Ok(Json(Progress {
        solved: by_difficulty.iter().map(|group| group.solved).sum(),
        total: by_difficulty.iter().map(|group| group.total).sum(),
        submissions,
        accepted,
        by_difficulty,
        by_tag,
        activity,
    }))
}
