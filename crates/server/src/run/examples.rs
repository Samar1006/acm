//! The Tests button: run the current code against a problem's public examples.
use axum::{async_trait, Extension, Json};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use shared::models::{
    forms::SubmitJob,
    language::Language,
    runner::{RunnerError, RunnerResponse},
    test::Test,
};
use sqlx::SqlitePool;
use tokio::sync::broadcast::Sender;

use crate::{
    auth::Claims,
    error::ServerError,
    problems::extras::{example_rows, visible_problem},
    ws::BroadcastMessage,
};

use super::{add_job, JobMap, JobQueue, JobStatus, Queueable};

/// Examples are small, so a run that takes longer than this is stuck.
const EXAMPLE_DEADLINE_SECONDS: u64 = 30;

/// Identifies exact source text, so any edit invalidates an earlier pass.
pub(crate) fn code_hash(code: &str) -> String {
    Sha256::digest(code.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Whether this exact code last passed the problem's examples.
pub(crate) async fn has_passed(
    pool: &SqlitePool,
    user_id: i64,
    problem_id: i64,
    language: Language,
    code: &str,
) -> Result<bool, ServerError> {
    let (passed,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM test_passes WHERE user_id = ? AND problem_id = ? AND language = ? AND code_hash = ?)",
    )
    .bind(user_id)
    .bind(problem_id)
    .bind(language)
    .bind(code_hash(code))
    .fetch_one(pool)
    .await
    .map_err(|error| {
        log::error!("error reading test pass: {error}");
        ServerError::InternalError
    })?;
    Ok(passed)
}

#[derive(Deserialize)]
pub struct ExamplesForm {
    #[serde(default)]
    pub language: Language,
    pub problem_id: i64,
    pub implementation: String,
}

/// Runs like a submission but stores no submission and no test results.
pub struct ExamplesJob(SubmitJob);

pub async fn examples(
    Extension(pool): Extension<SqlitePool>,
    Extension(job_queue): Extension<JobQueue>,
    Extension(job_map): Extension<JobMap>,
    Extension(broadcast): Extension<Sender<BroadcastMessage>>,
    claims: Claims,
    Json(form): Json<ExamplesForm>,
) -> Result<Json<JobStatus>, ServerError> {
    claims.validate_logged_in()?;
    visible_problem(&pool, form.problem_id, &claims).await?;

    let tests = example_rows(&pool, form.problem_id)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Test {
                id: row.example_number,
                index: row.example_number,
                max_fuel: None,
                input: serde_json::from_str(&row.input)?,
                expected_output: serde_json::from_str(&row.expected_output)?,
            })
        })
        .collect::<Result<Vec<_>, serde_json::Error>>()
        .map_err(|error| {
            log::error!("stored example is not valid JSON: {error}");
            ServerError::InternalError
        })?;
    if tests.is_empty() {
        return Err(ServerError::NotFound);
    }
    super::check_signature(form.language, tests.iter().map(|test| &test.input))?;

    let queue_item = Box::new(ExamplesJob(SubmitJob {
        language: form.language,
        problem_id: form.problem_id,
        user_id: claims.user_id,
        implementation: form.implementation,
        runtime_multiplier: None,
        tests,
        run_all: true,
        deadline_seconds: Some(EXAMPLE_DEADLINE_SECONDS),
    }));
    let job = add_job(claims.user_id, job_queue, job_map, queue_item, broadcast).await?;
    Ok(Json(job))
}

#[async_trait]
impl Queueable for ExamplesJob {
    async fn run(
        &self,
        ramiel_url: &str,
        pool: &SqlitePool,
        _broadcast: &Sender<BroadcastMessage>,
    ) -> Result<Value, ServerError> {
        let job = &self.0;
        let response = Client::new()
            .post(format!("{ramiel_url}/run/{}", job.language.route()))
            .json(job)
            .send()
            .await
            .map_err(|error| {
                log::error!("error fetching from ramiel: {error}");
                ServerError::InternalError
            })?;
        let result: Result<RunnerResponse, RunnerError> =
            response.json().await.map_err(|error| {
                log::error!("error decoding example results from ramiel: {error}");
                ServerError::InternalError
            })?;
        let mut result = result?;
        result.tests.sort_by_key(|test| test.index);

        if result.passed && result.tests.len() == job.tests.len() {
            sqlx::query(
                r#"INSERT INTO test_passes (user_id, problem_id, language, code_hash) VALUES (?, ?, ?, ?)
                ON CONFLICT (user_id, problem_id, language)
                DO UPDATE SET code_hash = excluded.code_hash, time = datetime('now')"#,
            )
            .bind(job.user_id)
            .bind(job.problem_id)
            .bind(job.language)
            .bind(code_hash(&job.implementation))
            .execute(pool)
            .await
            .map_err(|error| {
                log::error!("error recording test pass: {error}");
                ServerError::InternalError
            })?;
        }

        serde_json::to_value(result).map_err(|error| {
            log::error!("error serializing example results: {error}");
            ServerError::InternalError
        })
    }

    fn info(&self) -> String {
        format!(
            "ExamplesJob for problem {} run by user {}",
            self.0.problem_id, self.0.user_id
        )
    }

    fn job_type(&self) -> String {
        "ExamplesJob".to_string()
    }

    fn problem_id(&self) -> i64 {
        self.0.problem_id
    }
}
