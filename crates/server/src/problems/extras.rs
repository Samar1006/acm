//! Examples, tags, bookmarks, and saved drafts for a problem.
use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::models::language::{Language, RustSignature};
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};

use crate::{auth::Claims, error::ServerError};

const MAX_DRAFT_BYTES: usize = 200_000;
const MAX_TAGS: usize = 12;
const MAX_TAG_CHARS: usize = 40;

fn internal(error: sqlx::Error) -> ServerError {
    log::error!("{error}");
    ServerError::InternalError
}

/// Fails with NotFound unless the caller may see the problem. Returns its
/// competition and whether the Tests step is required before submitting.
pub(crate) async fn visible_problem(
    pool: &SqlitePool,
    problem_id: i64,
    claims: &Claims,
) -> Result<(Option<i64>, bool), ServerError> {
    let (visible, competition_id, require_tests): (bool, Option<i64>, Option<bool>) =
        sqlx::query_as("SELECT visible, competition_id, require_tests FROM problems WHERE id = ?")
            .bind(problem_id)
            .fetch_one(pool)
            .await
            .map_err(|_| ServerError::NotFound)?;
    if !visible && claims.validate_officer().is_err() {
        return Err(ServerError::NotFound);
    }
    // Competition problems skip the Tests step unless an officer turns it on.
    Ok((
        competition_id,
        require_tests.unwrap_or(competition_id.is_none()),
    ))
}

#[derive(FromRow)]
pub(crate) struct ExampleRow {
    pub example_number: i64,
    pub input: String,
    pub expected_output: String,
    pub explanation: String,
}

pub(crate) async fn example_rows(
    pool: &SqlitePool,
    problem_id: i64,
) -> Result<Vec<ExampleRow>, ServerError> {
    sqlx::query_as(
        "SELECT example_number, input, expected_output, explanation FROM problem_examples WHERE problem_id = ? ORDER BY example_number",
    )
    .bind(problem_id)
    .fetch_all(pool)
    .await
    .map_err(internal)
}

#[derive(Serialize)]
pub struct Example {
    index: i64,
    input: Value,
    expected_output: Value,
    explanation: String,
}

#[derive(Serialize)]
pub struct Examples {
    /// Argument names from the C++ starter code, when it can be parsed.
    parameter_names: Vec<String>,
    examples: Vec<Example>,
}

pub async fn examples(
    Path(problem_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
) -> Result<Json<Examples>, ServerError> {
    visible_problem(&pool, problem_id, &claims).await?;
    let (template,): (String,) = sqlx::query_as("SELECT template FROM problems WHERE id = ?")
        .bind(problem_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| ServerError::NotFound)?;
    let examples = example_rows(&pool, problem_id)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Example {
                index: row.example_number,
                input: serde_json::from_str(&row.input)?,
                expected_output: serde_json::from_str(&row.expected_output)?,
                explanation: row.explanation,
            })
        })
        .collect::<Result<_, serde_json::Error>>()
        .map_err(|error| {
            log::error!("stored example is not valid JSON: {error}");
            ServerError::InternalError
        })?;
    Ok(Json(Examples {
        parameter_names: RustSignature::parameter_names_from_cpp(&template),
        examples,
    }))
}

#[derive(Serialize)]
pub struct Meta {
    tags: Vec<String>,
    require_tests: bool,
    /// The stored choice behind `require_tests`: "default", "on", or "off".
    require_tests_setting: &'static str,
    example_count: i64,
    bookmarked: bool,
    solved: bool,
}

pub async fn meta(
    Path(problem_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
) -> Result<Json<Meta>, ServerError> {
    let (_, require_tests) = visible_problem(&pool, problem_id, &claims).await?;
    let tags: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM problem_tags WHERE problem_id = ? ORDER BY name")
            .bind(problem_id)
            .fetch_all(&pool)
            .await
            .map_err(internal)?;
    let (setting, example_count, bookmarked, solved): (Option<bool>, i64, bool, bool) = sqlx::query_as(
        r#"SELECT
            (SELECT require_tests FROM problems WHERE id = ?1),
            (SELECT COUNT(*) FROM problem_examples WHERE problem_id = ?1),
            EXISTS (SELECT 1 FROM bookmarks WHERE problem_id = ?1 AND user_id = ?2),
            EXISTS (SELECT 1 FROM submissions WHERE problem_id = ?1 AND user_id = ?2 AND success = true)"#,
    )
    .bind(problem_id)
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(internal)?;
    Ok(Json(Meta {
        tags: tags.into_iter().map(|(name,)| name).collect(),
        // Without examples there is nothing to run, so nothing is required.
        require_tests: require_tests && example_count > 0,
        require_tests_setting: match setting {
            Some(true) => "on",
            Some(false) => "off",
            None => "default",
        },
        example_count,
        bookmarked,
        solved,
    }))
}

#[derive(Serialize, FromRow)]
pub struct TagCount {
    name: String,
    count: i64,
}

/// Every tag in use on problems the caller can browse.
pub async fn tags(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<Vec<TagCount>>, ServerError> {
    sqlx::query_as(
        r#"SELECT problem_tags.name AS name, COUNT(*) AS count
        FROM problem_tags JOIN problems ON problems.id = problem_tags.problem_id
        WHERE problems.visible = true AND problems.competition_id IS NULL
        GROUP BY problem_tags.name
        ORDER BY problem_tags.name"#,
    )
    .fetch_all(&pool)
    .await
    .map(Json)
    .map_err(internal)
}

#[derive(Deserialize)]
pub struct BookmarkForm {
    bookmarked: bool,
}

pub async fn bookmark(
    Path(problem_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
    Json(form): Json<BookmarkForm>,
) -> Result<(), ServerError> {
    claims.validate_logged_in()?;
    visible_problem(&pool, problem_id, &claims).await?;
    let query = if form.bookmarked {
        "INSERT OR IGNORE INTO bookmarks (user_id, problem_id) VALUES (?, ?)"
    } else {
        "DELETE FROM bookmarks WHERE user_id = ? AND problem_id = ?"
    };
    sqlx::query(query)
        .bind(claims.user_id)
        .bind(problem_id)
        .execute(&pool)
        .await
        .map_err(internal)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct DraftQuery {
    #[serde(default)]
    language: Language,
}

#[derive(Serialize, FromRow)]
pub struct Draft {
    code: String,
}

pub async fn draft(
    Path(problem_id): Path<i64>,
    Query(query): Query<DraftQuery>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
) -> Result<Json<Option<Draft>>, ServerError> {
    claims.validate_logged_in()?;
    sqlx::query_as("SELECT code FROM drafts WHERE user_id = ? AND problem_id = ? AND language = ?")
        .bind(claims.user_id)
        .bind(problem_id)
        .bind(query.language)
        .fetch_optional(&pool)
        .await
        .map(Json)
        .map_err(internal)
}

#[derive(Deserialize)]
pub struct DraftForm {
    #[serde(default)]
    language: Language,
    /// `null` discards the saved draft.
    code: Option<String>,
}

pub async fn save_draft(
    Path(problem_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
    Json(form): Json<DraftForm>,
) -> Result<(), ServerError> {
    claims.validate_logged_in()?;
    visible_problem(&pool, problem_id, &claims).await?;
    match form.code {
        Some(code) if code.len() > MAX_DRAFT_BYTES => return Err(ServerError::PermissionDenied),
        Some(code) => sqlx::query(
            r#"INSERT INTO drafts (user_id, problem_id, language, code) VALUES (?, ?, ?, ?)
            ON CONFLICT (user_id, problem_id, language)
            DO UPDATE SET code = excluded.code, update_dt = datetime('now')"#,
        )
        .bind(claims.user_id)
        .bind(problem_id)
        .bind(form.language)
        .bind(code),
        None => {
            sqlx::query("DELETE FROM drafts WHERE user_id = ? AND problem_id = ? AND language = ?")
                .bind(claims.user_id)
                .bind(problem_id)
                .bind(form.language)
        }
    }
    .execute(&pool)
    .await
    .map_err(internal)?;
    Ok(())
}

/// An example is one of the problem's own visible tests plus an explanation,
/// so its expected output always comes from the reference solution.
#[derive(Deserialize, Clone)]
pub struct ExampleForm {
    pub test_number: i64,
    #[serde(default)]
    pub explanation: String,
}

pub(crate) async fn replace_tags(
    tx: &mut Transaction<'_, Sqlite>,
    problem_id: i64,
    tags: &[String],
) -> Result<(), ServerError> {
    sqlx::query("DELETE FROM problem_tags WHERE problem_id = ?")
        .bind(problem_id)
        .execute(&mut **tx)
        .await
        .map_err(internal)?;
    let mut seen = Vec::new();
    for tag in tags
        .iter()
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
    {
        if tag.chars().count() > MAX_TAG_CHARS || tag.contains(',') || seen.len() >= MAX_TAGS {
            return Err(crate::error::FormValidationError::InvalidField("tags".into()).into());
        }
        if seen.contains(&tag) {
            continue;
        }
        seen.push(tag);
        sqlx::query("INSERT INTO problem_tags (problem_id, name) VALUES (?, ?)")
            .bind(problem_id)
            .bind(tag)
            .execute(&mut **tx)
            .await
            .map_err(internal)?;
    }
    Ok(())
}

pub(crate) async fn replace_examples(
    tx: &mut Transaction<'_, Sqlite>,
    problem_id: i64,
    examples: &[ExampleForm],
) -> Result<(), ServerError> {
    sqlx::query("DELETE FROM problem_examples WHERE problem_id = ?")
        .bind(problem_id)
        .execute(&mut **tx)
        .await
        .map_err(internal)?;
    for (number, example) in examples.iter().enumerate() {
        // Hidden tests must never be published as examples.
        let copied = sqlx::query(
            r#"INSERT INTO problem_examples (problem_id, example_number, input, expected_output, explanation)
            SELECT problem_id, ?, input, expected_output, ?
            FROM tests WHERE problem_id = ? AND test_number = ? AND hidden = false"#,
        )
        .bind(number as i64)
        .bind(example.explanation.trim())
        .bind(problem_id)
        .bind(example.test_number)
        .execute(&mut **tx)
        .await
        .map_err(internal)?;
        if copied.rows_affected() != 1 {
            return Err(crate::error::FormValidationError::InvalidField("examples".into()).into());
        }
    }
    Ok(())
}
