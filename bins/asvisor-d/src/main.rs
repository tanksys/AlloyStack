use std::time::SystemTime;

use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use libasvisor::{
    isolation::{config::IsolationConfig, Isolation},
    logger,
};
use serde::Deserialize;

type AppResult<T> = Result<T, AppError>;
struct AppError(String);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}
impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        AppError(value.to_string())
    }
}

#[derive(Deserialize)]
struct TrigeWorkflowReq {
    isol_name: String,
}

async fn trige_workflow_handler(
    Query(TrigeWorkflowReq { mut isol_name }): Query<TrigeWorkflowReq>,
) -> AppResult<String> {
    log::info!("trige_workflow_handler: isol_name={}", isol_name);
    if !isol_name.ends_with(".json") {
        isol_name += ".json"
    };
    let config = IsolationConfig::from_file(isol_name.into())
        .map_err(|e| AppError(format!("load config file failed: {}", e)))?;

    // info!("preload?:{}", args.preload);
    let isol = Isolation::new(&config);
    tokio::task::spawn_blocking(move || isol.run())
        .await
        .unwrap()
        .map_err(|e| {
            let err_msg = format!("isolation user function error: {}", e);
            logger::error!("{}", err_msg);
            AppError(err_msg)
        })?;

    Ok("ok".to_owned())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    logger::init();
    let start = SystemTime::now();

    let app = Router::new().route("/workflow", get(trige_workflow_handler));

    let addr = "0.0.0.0:8000";
    let server = axum::Server::bind(&addr.parse().unwrap()).serve(app.into_make_service());

    log::info!(
        "listenning on: {}, init time: {}us",
        addr,
        SystemTime::now().duration_since(start).unwrap().as_micros()
    );
    server.await.unwrap();

    Ok(())
}
