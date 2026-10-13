use axum::{extract::Path, Extension, Json};

use serde::Deserialize;
use sqlx::SqlitePool;

use super::{
    extras::{replace_examples, replace_tags, ExampleForm},
    Difficulty,
};
use crate::{auth::Claims, error::ServerError};

#[derive(Deserialize)]
pub struct EditForm {
    title: String,
    description: String,
    template: String,
    runtime_multiplier: f64,
    difficulty: Difficulty,
    visible: bool,
    /// Omitted fields are left unchanged.
    tags: Option<Vec<String>>,
    examples: Option<Vec<ExampleForm>>,
    /// "default" (required unless in a competition), "on", or "off".
    require_tests: Option<String>,
}

pub async fn edit(
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
    Path(id): Path<i64>,
    Json(form): Json<EditForm>,
) -> Result<(), ServerError> {
    claims.validate_officer()?;

    log::info!("User {} editing problem {}", claims.user_id, id);

    sqlx::query!(
        r#"
        UPDATE problems SET
        title = ?,
        description = ?,
        difficulty = ?,
        visible = ?,
        template = ?,
        runtime_multiplier = ?
        WHERE id = ?
        "#,
        form.title,
        form.description,
        form.difficulty,
        form.visible,
        form.template,
        form.runtime_multiplier,
        id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        log::error!("{e}");
        ServerError::InternalError
    })?;

    let internal = |e: sqlx::Error| {
        log::error!("{e}");
        ServerError::InternalError
    };
    let mut tx = pool.begin().await.map_err(internal)?;
    if let Some(tags) = &form.tags {
        replace_tags(&mut tx, id, tags).await?;
    }
    if let Some(examples) = &form.examples {
        replace_examples(&mut tx, id, examples).await?;
    }
    if let Some(require_tests) = &form.require_tests {
        let value = match require_tests.as_str() {
            "on" => Some(true),
            "off" => Some(false),
            _ => None,
        };
        sqlx::query("UPDATE problems SET require_tests = ? WHERE id = ?")
            .bind(value)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
    }
    tx.commit().await.map_err(internal)?;

    Ok(())
}
