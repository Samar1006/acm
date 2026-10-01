use axum::{extract::Path, Extension, Json};
use shared::models::language::RustSignature;
use sqlx::SqlitePool;

use crate::{auth::Claims, error::ServerError};

use super::{Difficulty, Problem};

pub async fn rust_template(
    Path(problem_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
) -> Result<Json<String>, ServerError> {
    let (visible, template): (bool, String) =
        sqlx::query_as("SELECT visible, template FROM problems WHERE id = ?")
            .bind(problem_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| ServerError::NotFound)?;
    if !visible && claims.validate_officer().is_err() {
        return Err(ServerError::NotFound);
    }
    let signature = crate::run::rust_signature(&pool, problem_id, None).await?;
    let names = RustSignature::parameter_names_from_cpp(&template);
    Ok(Json(signature.template_with_names(&names)))
}

pub async fn problem(
    Path(problem_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
) -> Result<Json<Problem>, ServerError> {
    let problem = sqlx::query_as!(
        Problem,
        r#"
        SELECT
            id,
            title,
            description,
            runner,
            template,
            competition_id,
            visible,
            runtime_multiplier,
            difficulty as "difficulty: Difficulty"
        FROM
            problems
        WHERE
            id = ?
        "#,
        problem_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| ServerError::NotFound)?;

    if problem.visible || claims.validate_officer().is_ok() {
        Ok(Json(problem))
    } else {
        Err(ServerError::NotFound)
    }
}
