// These tests mock Ramiel HTTP responses; they do not run user code.
use super::*;
use shared::models::{
    forms::{CustomInputJob, SubmitJob},
    language::Language,
    runner::RunnerResponse,
};
use wasm_memory::{
    ContainerVariant, ContainerVariantType, FunctionType, FunctionValue, WasmFunctionCall,
};

fn input() -> WasmFunctionCall {
    WasmFunctionCall::new(
        "max_profit",
        vec![FunctionValue::Int(ContainerVariant::List(vec![1, 2]))],
        FunctionType::Int(ContainerVariantType::Single),
    )
}

async fn pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO users (id,name,username,discord_id) VALUES (1,'test','test','test')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO problems (id,title,description,runner,reference,template,visible) VALUES (1,'test','','','','int max_profit(vector<int> prices) {\n}\n',1)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO tests (problem_id,test_number,input,expected_output) VALUES (1,0,?,'null')",
    )
    .bind(serde_json::to_string(&input()).unwrap())
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn python_jobs_use_python_routes_and_keep_their_language() {
    let pool = pool().await;
    let app = axum::Router::new()
        .route(
            "/run/python",
            axum::routing::post(|| async { Json(Ok::<_, RunnerError>(RunnerResponse::default())) }),
        )
        .route(
            "/custom-input/python",
            axum::routing::post(|| async {
                Json(Ok::<_, RunnerError>(
                    serde_json::json!({"custom": "python"}),
                ))
            }),
        );
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service()),
    );
    let (broadcast, _) = broadcast::channel(8);

    let job: SubmitJob = serde_json::from_value(serde_json::json!({
        "language": "python",
        "problem_id": 1,
        "user_id": 1,
        "implementation": "def max_profit(prices):\n    return 0\n",
        "tests": [],
    }))
    .unwrap();
    assert_eq!(job.language, Language::Python);
    let result = job.run(&url, &pool, &broadcast).await.unwrap();
    assert_eq!(result["language"], "python");
    // The widened database constraint keeps earlier rows and accepts Python.
    let stored: crate::submissions::Submission =
        sqlx::query_as("SELECT * FROM submissions WHERE id=?")
            .bind(result["id"].as_i64().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored.language, Language::Python);
    assert_eq!(stored.code, job.implementation);
    assert!(sqlx::query(
        "INSERT INTO submissions (problem_id,user_id,success,runtime,code,language) VALUES (1,1,0,0,'','java')"
    )
    .execute(&pool)
    .await
    .is_err());

    let custom: CustomInputJob = serde_json::from_value(serde_json::json!({
        "language": "python",
        "problem_id": 1,
        "user_id": 1,
        "implementation": "",
        "reference": "",
        "input": input(),
    }))
    .unwrap();
    assert_eq!(
        custom.run(&url, &pool, &broadcast).await.unwrap()["custom"],
        "python"
    );
    server.abort();
}

#[tokio::test]
async fn python_custom_inputs_must_match_the_stored_signature() {
    let pool = pool().await;
    assert!(python_signature(&pool, 1, Some(&input())).await.is_ok());
    let mut different = input();
    different.arguments[0] = FunctionValue::Int(ContainerVariant::Single(1));
    assert!(python_signature(&pool, 1, Some(&different)).await.is_err());
    assert!(python_signature(&pool, 2, None).await.is_err());
}
