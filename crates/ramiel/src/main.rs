use std::{path::PathBuf, time::Duration};

use shared::models::{
    forms::{CustomInputJob, GenerateTestsJob, SubmitJob},
    runner::{CustomInputResponse, RunnerError, RunnerResponse},
    test::Test,
};

use actix_web::{get, middleware::Logger, post, web, web::Json, App, HttpResponse, HttpServer};

mod runners;

use clap::Parser;
use runners::{CPlusPlus, Python, Runner, Rust, WasmRuntime};

const RUN_TIMEOUT_MESSAGE: &str = "The tests took too long to run. (process killed)";

/// Submissions get 360 seconds; a job may ask for less, never more.
fn run_deadline(seconds: Option<u64>) -> tokio::time::Instant {
    tokio::time::Instant::now() + Duration::from_secs(seconds.map_or(360, |s| s.clamp(1, 360)))
}

#[get("/healthz")]
async fn healthz() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/run/c++")]
async fn cplusplus_run(
    runner: web::Data<CPlusPlus>,
    form: Json<SubmitJob>,
) -> Json<Result<RunnerResponse, RunnerError>> {
    let form = form.into_inner();
    let seconds = form.deadline_seconds;
    Json(
        runner
            .run_tests(form, run_deadline(seconds), RUN_TIMEOUT_MESSAGE)
            .await,
    )
}

#[post("/generate-tests/c++")]
async fn cplusplus_generate_tests(
    runner: web::Data<CPlusPlus>,
    form: Json<GenerateTestsJob>,
) -> Json<Result<Vec<Test>, RunnerError>> {
    Json(
        runner
            .generate_tests(
                form.into_inner(),
                tokio::time::Instant::now() + Duration::from_secs(120),
                RUN_TIMEOUT_MESSAGE,
            )
            .await,
    )
}

#[post("/custom-input/c++")]
async fn cplusplus_custom_input(
    runner: web::Data<CPlusPlus>,
    form: Json<CustomInputJob>,
) -> Json<Result<CustomInputResponse, RunnerError>> {
    Json(
        runner
            .run_custom_input(
                form.into_inner(),
                tokio::time::Instant::now() + Duration::from_secs(60),
                RUN_TIMEOUT_MESSAGE,
            )
            .await,
    )
}

#[post("/run/rust")]
async fn rust_run(
    runner: web::Data<Rust>,
    form: Json<SubmitJob>,
) -> Json<Result<RunnerResponse, RunnerError>> {
    let form = form.into_inner();
    let seconds = form.deadline_seconds;
    Json(
        runner
            .run_tests(form, run_deadline(seconds), RUN_TIMEOUT_MESSAGE)
            .await,
    )
}

#[post("/custom-input/rust")]
async fn rust_custom_input(
    runner: web::Data<Rust>,
    form: Json<CustomInputJob>,
) -> Json<Result<CustomInputResponse, RunnerError>> {
    Json(
        runner
            .run_custom_input(
                form.into_inner(),
                tokio::time::Instant::now() + Duration::from_secs(60),
                RUN_TIMEOUT_MESSAGE,
            )
            .await,
    )
}

#[post("/run/python")]
async fn python_run(
    runner: web::Data<Python>,
    form: Json<SubmitJob>,
) -> Json<Result<RunnerResponse, RunnerError>> {
    let form = form.into_inner();
    let seconds = form.deadline_seconds;
    Json(
        runner
            .run_tests(form, run_deadline(seconds), RUN_TIMEOUT_MESSAGE)
            .await,
    )
}

#[post("/custom-input/python")]
async fn python_custom_input(
    runner: web::Data<Python>,
    form: Json<CustomInputJob>,
) -> Json<Result<CustomInputResponse, RunnerError>> {
    Json(
        runner
            .run_custom_input(
                form.into_inner(),
                tokio::time::Instant::now() + Duration::from_secs(60),
                RUN_TIMEOUT_MESSAGE,
            )
            .await,
    )
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, env, long, default_value_t = 8082)]
    port: u16,

    #[arg(long, env, default_value = "127.0.0.1")]
    hostname: String,

    #[arg(
        long,
        env = "WASMTIME_CACHE_CONFIG",
        default_value = "./wasmtime-cache.toml"
    )]
    wasmtime_cache_config: PathBuf,

    /// Directory holding `python.wasm` and its `lib` standard library.
    #[arg(long, env = "ACM_PYTHON_HOME", default_value = "/opt/acm-python")]
    python_home: PathBuf,
}

fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));
    let args = Args::parse();
    runners::initialize_compiler_sandbox()?;
    let runtime = WasmRuntime::new(&args.wasmtime_cache_config).map_err(std::io::Error::other)?;
    let runner = CPlusPlus::new(runtime.clone());
    let rust = Rust::new(runtime.clone());
    let python = Python::new(runtime, &args.python_home);

    actix_web::rt::System::new().block_on(run_server(args, runner, rust, python))
}

async fn run_server(
    args: Args,
    runner: CPlusPlus,
    rust: Rust,
    python: Python,
) -> std::io::Result<()> {
    let json_cfg = web::JsonConfig::default()
        // 3mb limit
        .limit(100_000_000);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(json_cfg.clone())
            .app_data(web::Data::new(runner.clone()))
            .app_data(web::Data::new(rust.clone()))
            .app_data(web::Data::new(python.clone()))
            .service(rust_run)
            .service(rust_custom_input)
            .service(python_run)
            .service(python_custom_input)
            .service(healthz)
            .service(cplusplus_run)
            .service(cplusplus_generate_tests)
            .service(cplusplus_custom_input)
    })
    .bind(&format!("{}:{}", args.hostname, args.port))?
    .run()
    .await
}
