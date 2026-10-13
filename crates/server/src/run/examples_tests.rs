// These tests mock Ramiel HTTP responses; they do not run user code.
use super::*;
use crate::auth::Auth;
use axum::extract::Query;
use shared::models::{
    forms::SubmitJob,
    runner::RunnerResponse,
    test::{Test, TestResult},
};
use wasm_memory::{
    ContainerVariant, ContainerVariantType, FunctionType, FunctionValue, WasmFunctionCall,
};

fn call(value: i32) -> WasmFunctionCall {
    WasmFunctionCall::new(
        "double",
        vec![FunctionValue::Int(ContainerVariant::Single(value))],
        FunctionType::Int(ContainerVariantType::Single),
    )
}

fn claims(user_id: i64, auth: Auth) -> Claims {
    Claims {
        user_id,
        auth,
        exp: 0,
    }
}

/// Two users, one practice problem with three tests (one hidden), and two
/// published examples.
async fn pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
    for id in [1, 2] {
        sqlx::query("INSERT INTO users (id,name,username,discord_id) VALUES (?,?,?,?)")
            .bind(id)
            .bind(format!("user{id}"))
            .bind(format!("user{id}"))
            .bind(format!("discord{id}"))
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO problems (id,title,description,runner,reference,template,visible,difficulty) VALUES (1,'Double','','','','int double(int value) {\n}\n',1,'Easy')")
        .execute(&pool)
        .await
        .unwrap();
    for (number, hidden) in [(0, false), (1, false), (2, true)] {
        sqlx::query("INSERT INTO tests (problem_id,test_number,input,expected_output,hidden) VALUES (1,?,?,?,?)")
            .bind(number)
            .bind(serde_json::to_string(&call(number)).unwrap())
            .bind(serde_json::to_string(&FunctionValue::Int(ContainerVariant::Single(number * 2))).unwrap())
            .bind(hidden)
            .execute(&pool)
            .await
            .unwrap();
    }
    edit(
        &pool,
        serde_json::json!({
            "tags": ["Math", " Arrays ", "Math"],
            "examples": [
                {"test_number": 0, "explanation": "Zero doubled is zero."},
                {"test_number": 1, "explanation": "One doubled is two."},
            ],
        }),
    )
    .await
    .unwrap();
    pool
}

/// Edits problem 1 through the real officer endpoint.
async fn edit(pool: &SqlitePool, extra: serde_json::Value) -> Result<(), ServerError> {
    let mut form = serde_json::json!({
        "title": "Double",
        "description": "Return twice the value.",
        "template": "int double(int value) {\n}\n",
        "runtime_multiplier": 1.0,
        "difficulty": "Easy",
        "visible": true,
    });
    form.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    crate::problems::edit_for_tests(pool, claims(1, Auth::Officer), form).await
}

/// A fake runner that passes or fails every test it is given and records
/// each job it receives.
fn runner(pass: bool) -> (String, Arc<std::sync::Mutex<Vec<SubmitJob>>>) {
    let jobs = Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen = jobs.clone();
    let app = axum::Router::new().route(
        "/run/c++",
        axum::routing::post(move |Json(job): Json<SubmitJob>| {
            let seen = seen.clone();
            async move {
                let tests: Vec<TestResult> = job
                    .tests
                    .iter()
                    .cloned()
                    .map(|test: Test| {
                        let mut output = test.expected_output.clone();
                        if !pass {
                            output = FunctionValue::Int(ContainerVariant::Single(-1));
                        }
                        test.make_result(output, 7)
                    })
                    .collect();
                seen.lock().unwrap().push(job);
                Json(Ok::<_, RunnerError>(RunnerResponse {
                    passed: tests.iter().all(|test| test.success),
                    runtime: 14,
                    tests,
                }))
            }
        }),
    );
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service()),
    );
    (url, jobs)
}

type Queue = (
    JobQueue,
    mpsc::UnboundedReceiver<(u64, JobQueueItem)>,
    JobMap,
    broadcast::Sender<BroadcastMessage>,
);

fn queue() -> Queue {
    let (queue, receiver) = mpsc::unbounded_channel();
    let (broadcast, _) = broadcast::channel(8);
    (
        queue,
        receiver,
        Arc::new(RwLock::new(HashMap::new())),
        broadcast,
    )
}

async fn run_examples(
    pool: &SqlitePool,
    url: &str,
    user: Claims,
    code: &str,
) -> Result<serde_json::Value, ServerError> {
    let (queue, mut receiver, map, broadcast) = queue();
    let _queued = examples::examples(
        Extension(pool.clone()),
        Extension(queue),
        Extension(map),
        Extension(broadcast.clone()),
        user,
        Json(
            serde_json::from_value(serde_json::json!({"problem_id": 1, "implementation": code}))
                .unwrap(),
        ),
    )
    .await?;
    let (_, job) = receiver.try_recv().unwrap();
    job.run(url, pool, &broadcast).await
}

async fn submit(pool: &SqlitePool, user: Claims, code: &str) -> Result<JobQueueItem, ServerError> {
    let (queue, mut receiver, map, broadcast) = queue();
    let _queued = submit::submit(
        Extension(pool.clone()),
        Extension(queue),
        Extension(map),
        Extension(broadcast),
        user,
        Json(
            serde_json::from_value(serde_json::json!({"problem_id": 1, "implementation": code}))
                .unwrap(),
        ),
    )
    .await?;
    Ok(receiver.try_recv().unwrap().1)
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_as::<_, (i64,)>(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
        .0
}

#[tokio::test]
async fn submit_requires_passing_examples_with_the_same_code() {
    let pool = pool().await;
    let (passing, jobs) = runner(true);
    let (failing, _) = runner(false);
    let member = || claims(2, Auth::Member);

    // Nothing has been tested yet.
    assert!(matches!(
        submit(&pool, member(), "v1").await,
        Err(ServerError::TestsRequired)
    ));

    // A failed example run reports both examples and unlocks nothing.
    let result = run_examples(&pool, &failing, member(), "v1").await.unwrap();
    assert_eq!(result["passed"], false);
    assert_eq!(result["tests"].as_array().unwrap().len(), 2);
    assert!(matches!(
        submit(&pool, member(), "v1").await,
        Err(ServerError::TestsRequired)
    ));

    // Only the published examples are sent, every one is run, and the run is short.
    let result = run_examples(&pool, &passing, member(), "v1").await.unwrap();
    assert_eq!(result["passed"], true);
    {
        let jobs = jobs.lock().unwrap();
        let job = jobs.last().unwrap();
        assert!(job.run_all && job.deadline_seconds.is_some());
        let sent: Vec<_> = job.tests.iter().map(|test| test.index).collect();
        assert_eq!(sent, [0, 1]);
    }

    // Example runs are not submissions.
    assert_eq!(count(&pool, "submissions").await, 0);
    assert_eq!(count(&pool, "test_results").await, 0);

    // The pass covers exactly that code, language, and user.
    assert!(submit(&pool, member(), "v1").await.is_ok());
    assert!(matches!(
        submit(&pool, member(), "v1 ").await,
        Err(ServerError::TestsRequired)
    ));
    assert!(matches!(
        submit(&pool, claims(1, Auth::Member), "v1").await,
        Err(ServerError::TestsRequired)
    ));
    let (queue, _receiver, map, broadcast) = queue();
    let rust = submit::submit(
        Extension(pool.clone()),
        Extension(queue),
        Extension(map),
        Extension(broadcast),
        member(),
        Json(
            serde_json::from_value(
                serde_json::json!({"problem_id": 1, "implementation": "v1", "language": "rust"}),
            )
            .unwrap(),
        ),
    )
    .await;
    assert!(matches!(rust, Err(ServerError::TestsRequired)));

    // Testing newer code replaces the earlier pass.
    run_examples(&pool, &passing, member(), "v2").await.unwrap();
    assert!(submit(&pool, member(), "v2").await.is_ok());
    assert!(matches!(
        submit(&pool, member(), "v1").await,
        Err(ServerError::TestsRequired)
    ));

    // Logged-out users cannot run examples; officers may submit untested code.
    assert!(
        run_examples(&pool, &passing, claims(-1, Auth::LoggedOut), "v1")
            .await
            .is_err()
    );
    assert!(submit(&pool, claims(1, Auth::Officer), "untested")
        .await
        .is_ok());

    // Officers can turn the requirement off and back on per problem.
    edit(&pool, serde_json::json!({"require_tests": "off"}))
        .await
        .unwrap();
    assert!(submit(&pool, member(), "untested").await.is_ok());
    edit(&pool, serde_json::json!({"require_tests": "on"}))
        .await
        .unwrap();
    assert!(submit(&pool, member(), "untested").await.is_err());

    // By default competition problems do not require it.
    edit(&pool, serde_json::json!({"require_tests": "default"}))
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO competitions (id,name,start,end) VALUES (1,'c','2020-01-01','2020-01-02')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE problems SET competition_id = 1")
        .execute(&pool)
        .await
        .unwrap();
    assert!(submit(&pool, member(), "untested").await.is_ok());
}

#[tokio::test]
async fn compilation_errors_are_stored_but_never_counted() {
    let pool = pool().await;
    let app = axum::Router::new().route(
        "/run/c++",
        axum::routing::post(|| async {
            Json(Err::<RunnerResponse, _>(RunnerError::CompilationError {
                diagnostics: vec![],
            }))
        }),
    );
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let broken = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service()),
    );
    let (passing, _) = runner(true);
    let (failing, _) = runner(false);
    let (broadcast, _) = broadcast::channel(8);
    let officer = || claims(1, Auth::Officer);

    for (url, verdict) in [
        (&broken, "CompilationError"),
        (&failing, "WrongAnswer"),
        (&passing, "Accepted"),
        (&broken, "CompilationError"),
    ] {
        let job = submit(&pool, officer(), "code").await.unwrap();
        let stored = job.run(url, &pool, &broadcast).await.unwrap();
        assert_eq!(stored["verdict"], verdict);
    }

    // The author still sees every attempt, including the compiler output.
    let Json(history) = crate::problems::history_for_tests(&pool, 1, officer())
        .await
        .unwrap();
    assert_eq!(history.as_array().unwrap().len(), 4);
    assert_eq!(history[0]["verdict"], "CompilationError");
    assert_eq!(history[0]["error"], "[]");

    // Public lists and statistics leave compilation errors out.
    let Json(progress) = crate::user::progress_for_tests(&pool, "user1")
        .await
        .unwrap();
    assert_eq!(progress["submissions"], 2);
    assert_eq!(progress["accepted"], 1);
    assert_eq!(progress["solved"], 1);
    assert_eq!(progress["total"], 1);
    assert_eq!(progress["by_difficulty"][0]["name"], "Easy");
    assert_eq!(progress["by_tag"].as_array().unwrap().len(), 2);
    assert_eq!(progress["by_tag"][1]["name"], "Math");
    assert_eq!(progress["by_tag"][1]["solved"], 1);
    assert_eq!(progress["activity"][0]["submissions"], 2);
    let Json(public) = crate::user::submissions_for_tests(&pool, "user1")
        .await
        .unwrap();
    let verdicts: Vec<_> = public
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["verdict"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(verdicts.len(), 2);
    assert!(!verdicts.contains(&"CompilationError".to_string()));

    // The other user has solved nothing.
    let Json(progress) = crate::user::progress_for_tests(&pool, "user2")
        .await
        .unwrap();
    assert_eq!(
        (
            progress["solved"].as_i64(),
            progress["submissions"].as_i64()
        ),
        (Some(0), Some(0))
    );
}

#[tokio::test]
async fn listing_filters_examples_bookmarks_and_drafts() {
    use crate::problems::extras;
    let pool = pool().await;
    sqlx::query("INSERT INTO problems (id,title,description,runner,reference,template,visible,difficulty) VALUES (2,'Alpha','','','','',1,'Hard'), (3,'Secret','','','','',0,'Easy')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO problem_tags (problem_id,name) VALUES (2,'Math'), (3,'Math')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO submissions (problem_id,user_id,success,runtime,code,verdict) VALUES (2,2,1,1,'','Accepted')")
        .execute(&pool)
        .await
        .unwrap();
    let member = || claims(2, Auth::Member);

    let list = |query: &'static str, user: Claims| {
        let pool = pool.clone();
        async move {
            let Json(value) = crate::problems::list_for_tests(&pool, query, user)
                .await
                .unwrap();
            value
                .as_array()
                .unwrap()
                .iter()
                .map(|problem| problem["title"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        }
    };
    assert_eq!(list("sort_by=Title", member()).await, ["Alpha", "Double"]);
    assert_eq!(
        list("sort_by=Difficulty", member()).await,
        ["Double", "Alpha"]
    );
    assert_eq!(list("tags=Math,Arrays", member()).await, ["Double"]);
    assert_eq!(list("tags=Math&difficulty=4", member()).await, ["Alpha"]);
    assert_eq!(list("status=solved", member()).await, ["Alpha"]);
    assert_eq!(list("status=unsolved", member()).await, ["Double"]);
    assert_eq!(
        list("status=unsolved&sort_by=Random&count=1", member()).await,
        ["Double"]
    );
    assert_eq!(list("query=alp", member()).await, ["Alpha"]);
    assert_eq!(
        list("query=alp&status=unsolved", member()).await,
        Vec::<String>::new()
    );
    // Logged-out visitors get the plain list; hidden problems stay hidden.
    assert_eq!(
        list(
            "status=solved&bookmarked=true&sort_by=Title",
            claims(-1, Auth::LoggedOut)
        )
        .await
        .len(),
        2
    );
    assert_eq!(
        list("sort_by=Title", claims(1, Auth::Officer)).await.len(),
        3
    );

    extras::bookmark(
        Path(1),
        Extension(pool.clone()),
        member(),
        Json(serde_json::from_value(serde_json::json!({"bookmarked": true})).unwrap()),
    )
    .await
    .unwrap();
    assert_eq!(list("bookmarked=true", member()).await, ["Double"]);
    assert!(extras::bookmark(
        Path(3),
        Extension(pool.clone()),
        member(),
        Json(serde_json::from_value(serde_json::json!({"bookmarked": true})).unwrap())
    )
    .await
    .is_err());
    let Json(listing) = crate::problems::list_for_tests(&pool, "sort_by=Title", member())
        .await
        .unwrap();
    assert_eq!(listing[1]["tags"], serde_json::json!(["Math", "Arrays"]));
    assert_eq!(listing[1]["bookmarked"], true);
    assert_eq!(listing[0]["solved"], true);

    // Examples carry argument names and never include the hidden test.
    let Json(examples) = extras::examples(Path(1), Extension(pool.clone()), member())
        .await
        .unwrap();
    let examples = serde_json::to_value(examples).unwrap();
    assert_eq!(examples["parameter_names"], serde_json::json!(["value"]));
    assert_eq!(examples["examples"].as_array().unwrap().len(), 2);
    assert_eq!(
        examples["examples"][1]["explanation"],
        "One doubled is two."
    );
    assert!(
        edit(&pool, serde_json::json!({"examples": [{"test_number": 2}]}))
            .await
            .is_err()
    );
    assert!(extras::examples(Path(3), Extension(pool.clone()), member())
        .await
        .is_err());
    let Json(meta) = extras::meta(Path(1), Extension(pool.clone()), member())
        .await
        .unwrap();
    let meta = serde_json::to_value(meta).unwrap();
    assert_eq!(meta["require_tests"], true);
    assert_eq!(meta["bookmarked"], true);
    assert_eq!(meta["tags"], serde_json::json!(["Arrays", "Math"]));
    let Json(tags) = extras::tags(Extension(pool.clone())).await.unwrap();
    assert_eq!(
        serde_json::to_value(tags).unwrap(),
        serde_json::json!([{"name": "Arrays", "count": 1}, {"name": "Math", "count": 2}])
    );

    // Drafts are private to each user and language, and can be discarded.
    let save = |user: Claims, body: serde_json::Value| {
        let pool = pool.clone();
        async move {
            extras::save_draft(
                Path(1),
                Extension(pool),
                user,
                Json(serde_json::from_value(body).unwrap()),
            )
            .await
        }
    };
    let load = |user: Claims, language: &'static str| {
        let pool = pool.clone();
        async move {
            let query =
                Query(serde_json::from_value(serde_json::json!({"language": language})).unwrap());
            let Json(draft) = extras::draft(Path(1), query, Extension(pool), user)
                .await
                .unwrap();
            serde_json::to_value(draft).unwrap()["code"]
                .as_str()
                .map(str::to_owned)
        }
    };
    save(
        member(),
        serde_json::json!({"language": "python", "code": "first"}),
    )
    .await
    .unwrap();
    save(
        member(),
        serde_json::json!({"language": "python", "code": "second"}),
    )
    .await
    .unwrap();
    assert_eq!(load(member(), "python").await.as_deref(), Some("second"));
    assert_eq!(load(member(), "cpp").await, None);
    assert_eq!(load(claims(1, Auth::Member), "python").await, None);
    assert!(save(
        claims(-1, Auth::LoggedOut),
        serde_json::json!({"code": "x"})
    )
    .await
    .is_err());
    assert!(
        save(member(), serde_json::json!({"code": "x".repeat(200_001)}))
            .await
            .is_err()
    );
    save(
        member(),
        serde_json::json!({"language": "python", "code": null}),
    )
    .await
    .unwrap();
    assert_eq!(load(member(), "python").await, None);
}

#[tokio::test]
async fn practice_fixtures_create_tagged_problems_with_published_examples() {
    use crate::problems::extras;
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
    let fixture = include_str!("../../../../docs/examples/practice-set-2/medium-unique-paths.json");
    let id = crate::problems::new_for_tests(&pool, claims(1, Auth::Officer), fixture)
        .await
        .unwrap();
    let Json(meta) = extras::meta(Path(id), Extension(pool.clone()), claims(2, Auth::Member))
        .await
        .unwrap();
    let meta = serde_json::to_value(meta).unwrap();
    assert_eq!(
        meta["tags"],
        serde_json::json!(["Dynamic Programming", "Math"])
    );
    assert_eq!(meta["example_count"], 2);
    assert_eq!(meta["require_tests"], true);
    let Json(examples) =
        extras::examples(Path(id), Extension(pool.clone()), claims(2, Auth::Member))
            .await
            .unwrap();
    let examples = serde_json::to_value(examples).unwrap();
    assert_eq!(examples["parameter_names"], serde_json::json!(["m", "n"]));
    assert_eq!(
        examples["examples"][0]["expected_output"],
        serde_json::json!({"Int": {"Single": 28}})
    );
    assert!(examples["examples"][1]["explanation"]
        .as_str()
        .unwrap()
        .contains("3 paths"));

    // A fixture that names a missing test as an example is rejected as a whole.
    let mut broken: serde_json::Value = serde_json::from_str(fixture).unwrap();
    broken["title"] = "Other".into();
    broken["examples"][0]["test_number"] = 999.into();
    assert!(
        crate::problems::new_for_tests(&pool, claims(1, Auth::Officer), &broken.to_string())
            .await
            .is_err()
    );
    assert_eq!(count(&pool, "problems").await, 1);
}
