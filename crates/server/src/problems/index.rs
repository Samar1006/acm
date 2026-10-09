use axum::{extract::Query, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};

use crate::{auth::Claims, error::ServerError, pagination::Pagination};

use super::Problem;

#[derive(Deserialize, Default)]
enum ProblemOrdering {
    #[default]
    Newest,
    Oldest,
    Title,
    Difficulty,
    Random,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum SolvedStatus {
    Solved,
    Unsolved,
}

#[derive(Deserialize)]
pub struct ProblemOptions {
    competition_id: Option<i64>,
    show_competition_problems: Option<bool>,

    // Bit 0: Easy, bit 1: Medium, bit 2: Hard
    difficulty: Option<u8>,

    query: Option<String>,

    /// Comma-separated tag names; a problem must have all of them.
    tags: Option<String>,

    /// Only for logged-in users; ignored otherwise.
    status: Option<SolvedStatus>,
    bookmarked: Option<bool>,

    #[serde(default)]
    sort_by: ProblemOrdering,
}

/// A problem with the listing details that depend on tags and the caller.
#[derive(Serialize)]
pub struct ProblemListing {
    #[serde(flatten)]
    problem: Problem,
    tags: Vec<String>,
    solved: bool,
    bookmarked: bool,
}

const SOLVED: &str = "(EXISTS (SELECT 1 FROM submissions WHERE submissions.problem_id = problems.id AND submissions.success = true AND submissions.user_id = ";
const BOOKMARKED: &str =
    "(EXISTS (SELECT 1 FROM bookmarks WHERE bookmarks.problem_id = problems.id AND bookmarks.user_id = ";

pub async fn problems(
    Extension(pool): Extension<SqlitePool>,
    Query(options): Query<ProblemOptions>,
    Query(pagination): Query<Pagination<0, 10>>,
    claims: Claims,
) -> Result<Json<Vec<ProblemListing>>, ServerError> {
    let is_officer = claims.validate_officer().is_ok();
    let logged_in = claims.validate_logged_in().is_ok();
    let search = options.query.filter(|query| !query.trim().is_empty());
    let has_query = search.is_some();

    let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r#"SELECT
            problems.id,
            problems.title,
            problems.description,
            problems.runner,
            problems.template,
            problems.competition_id,
            problems.visible,
            problems.runtime_multiplier,
            problems.difficulty,
            (SELECT group_concat(name, char(31)) FROM problem_tags WHERE problem_tags.problem_id = problems.id) AS tags,
            "#,
    );
    builder
        .push(SOLVED)
        .push_bind(claims.user_id)
        .push(")) AS solved, ");
    builder
        .push(BOOKMARKED)
        .push_bind(claims.user_id)
        .push(")) AS bookmarked FROM problems ");

    if let Some(query) = search {
        // Quote the text so it is matched literally, as a title prefix.
        let prefix = format!("\"{}\"*", query.replace('"', "\"\""));
        builder
            .push("INNER JOIN (SELECT rowid, rank FROM problems_fts WHERE title MATCH ")
            .push_bind(prefix)
            .push(") search ON search.rowid = problems.id ");
    }

    builder.push("WHERE TRUE");

    if !is_officer {
        builder.push(" AND problems.visible = true");
    }

    if let Some(competition_id) = options.competition_id {
        builder
            .push(" AND problems.competition_id = ")
            .push_bind(competition_id);
    } else if !options.show_competition_problems.unwrap_or(false) && !has_query {
        builder.push(" AND problems.competition_id IS NULL");
    }

    let difficulties: Vec<&str> = [(1, "Easy"), (2, "Medium"), (4, "Hard")]
        .into_iter()
        .filter(|(bit, _)| options.difficulty.unwrap_or(0) & bit != 0)
        .map(|(_, name)| name)
        .collect();
    if !difficulties.is_empty() {
        builder.push(" AND problems.difficulty IN (");
        let mut list = builder.separated(", ");
        for difficulty in difficulties {
            list.push_bind(difficulty);
        }
        builder.push(")");
    }

    for tag in options
        .tags
        .iter()
        .flat_map(|tags| tags.split(','))
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .take(16)
    {
        builder
            .push(" AND EXISTS (SELECT 1 FROM problem_tags WHERE problem_tags.problem_id = problems.id AND problem_tags.name = ")
            .push_bind(tag.to_string())
            .push(")");
    }

    if logged_in {
        match options.status {
            Some(SolvedStatus::Solved) => {
                builder
                    .push(" AND ")
                    .push(SOLVED)
                    .push_bind(claims.user_id)
                    .push("))");
            }
            Some(SolvedStatus::Unsolved) => {
                builder
                    .push(" AND NOT ")
                    .push(SOLVED)
                    .push_bind(claims.user_id)
                    .push("))");
            }
            None => {}
        }
        if options.bookmarked.unwrap_or(false) {
            builder
                .push(" AND ")
                .push(BOOKMARKED)
                .push_bind(claims.user_id)
                .push("))");
        }
    }

    builder.push(match options.sort_by {
        ProblemOrdering::Random => " ORDER BY random()",
        ProblemOrdering::Title => " ORDER BY problems.title COLLATE NOCASE",
        ProblemOrdering::Difficulty => {
            " ORDER BY CASE problems.difficulty WHEN 'Easy' THEN 0 WHEN 'Medium' THEN 1 ELSE 2 END, problems.title COLLATE NOCASE"
        }
        _ if has_query => " ORDER BY rank",
        _ if options.competition_id.is_some() => " ORDER BY problems.title",
        ProblemOrdering::Oldest => " ORDER BY problems.create_dt ASC",
        ProblemOrdering::Newest => " ORDER BY problems.create_dt DESC",
    });

    builder
        .push(" LIMIT ")
        .push_bind(pagination.count.clamp(0, 100))
        .push(" OFFSET ")
        .push_bind(pagination.offset.max(0));

    let problems = builder
        .build()
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            log::error!("{e}");
            ServerError::InternalError
        })?
        .iter()
        .map(|row| ProblemListing {
            problem: Problem {
                id: row.get_unchecked("id"),
                competition_id: row.get_unchecked("competition_id"),
                title: row.get_unchecked("title"),
                description: row.get_unchecked("description"),
                runner: row.get_unchecked("runner"),
                runtime_multiplier: row.get_unchecked("runtime_multiplier"),
                template: row.get_unchecked("template"),
                visible: row.get_unchecked("visible"),
                difficulty: row.get_unchecked("difficulty"),
            },
            tags: row
                .get_unchecked::<Option<String>, _>("tags")
                .map(|tags| tags.split('\u{1f}').map(str::to_owned).collect())
                .unwrap_or_default(),
            solved: row.get_unchecked("solved"),
            bookmarked: row.get_unchecked("bookmarked"),
        })
        .collect();

    Ok(Json(problems))
}
