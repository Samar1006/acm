use axum::{extract::Path, Extension, Json};
use shared::models::language::RustSignature;
use sqlx::SqlitePool;

use crate::{auth::Claims, error::ServerError};

use super::{Difficulty, Problem};

/// Returns the C++ starter code of a problem the caller may see.
async fn visible_template(
    pool: &SqlitePool,
    problem_id: i64,
    claims: &Claims,
) -> Result<String, ServerError> {
    let (visible, template): (bool, String) =
        sqlx::query_as("SELECT visible, template FROM problems WHERE id = ?")
            .bind(problem_id)
            .fetch_one(pool)
            .await
            .map_err(|_| ServerError::NotFound)?;
    if !visible && claims.validate_officer().is_err() {
        return Err(ServerError::NotFound);
    }
    Ok(template)
}

pub async fn rust_template(
    Path(problem_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
) -> Result<Json<String>, ServerError> {
    let template = visible_template(&pool, problem_id, &claims).await?;
    let signature = crate::run::rust_signature(&pool, problem_id, None).await?;
    let names = RustSignature::parameter_names_from_cpp(&template);
    Ok(Json(signature.template_with_names(&names)))
}

pub async fn python_template(
    Path(problem_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    claims: Claims,
) -> Result<Json<String>, ServerError> {
    let template = visible_template(&pool, problem_id, &claims).await?;
    let signature = crate::run::python_signature(&pool, problem_id, None).await?;
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

#[cfg(test)]
mod template_tests {
    use super::*;
    use crate::auth::Auth;

    #[tokio::test]
    async fn python_template_uses_stored_tests_and_respects_visibility() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
        sqlx::query("INSERT INTO problems (id,title,description,runner,reference,template,visible) VALUES (1,'test','','','','int max_profit(vector<int> prices) {\n}\n',1)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO tests (problem_id,test_number,input,expected_output) VALUES (1,0,?,'null')")
            .bind(r#"{"name":"max_profit","arguments":[{"Int":{"List":[1,2]}}],"return_type":{"Int":"Single"}}"#)
            .execute(&pool)
            .await
            .unwrap();
        let claims = |auth| Claims {
            user_id: 1,
            auth,
            exp: 0,
        };
        let Json(template) =
            python_template(Path(1), Extension(pool.clone()), claims(Auth::Member))
                .await
                .unwrap();
        assert_eq!(
            template,
            "def max_profit(prices: list[int]) -> int:\n    # Return the answer.\n    return 0\n"
        );
        sqlx::query("UPDATE problems SET visible=0 WHERE id=1")
            .execute(&pool)
            .await
            .unwrap();
        assert!(matches!(
            python_template(Path(1), Extension(pool.clone()), claims(Auth::Member)).await,
            Err(ServerError::NotFound)
        ));
        assert!(
            python_template(Path(1), Extension(pool), claims(Auth::Officer))
                .await
                .is_ok()
        );
    }
}
