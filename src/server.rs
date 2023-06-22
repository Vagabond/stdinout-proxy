use super::*;
use axum::{extract::Query, http::{header, StatusCode}, response, routing::get, Extension, Router, response::{IntoResponse},};
use axum_auth::AuthBearer;
use axum::body::StreamBody;
use daemon_handle::DaemonHandle;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use std::io;

use futures_util::stream::{self, Stream};

pub type HandlerResult = std::result::Result<response::Json<Value>, (StatusCode, String)>;

#[derive(Debug, Clone, clap::Args)]
pub struct Server {}

impl Server {
    pub async fn run(self) -> Result {
        let daemon: Arc<DaemonHandle> = Arc::new(DaemonHandle::new().await?);

        let app = Router::new()
            .route("/v1/path", get(path_handler))
            .route("/v1/plot", get(plot_handler))
            .route("/v1/h3plot", get(h3plot_handler))
            .layer(Extension(daemon));

        let server_endpoint = std::env::var("PORT").unwrap_or("3000".to_string());
        println!("Binding to port {}...", server_endpoint);
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], server_endpoint.parse().unwrap()));
        tokio::select!(
            result = axum::Server::bind(&addr)
                .serve(app.into_make_service()) =>
                    result.map_err(|e| Error::Axum(e.into())),
        )
    }
}

impl From<Error> for HandlerResult {
    fn from(e: Error) -> Self {
        Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
}

pub async fn path_handler(
    Extension(daemon): Extension<Arc<DaemonHandle>>,
    //Extension(env_token): Extension<String>,
    query: Query<daemon_handle::PathParams>,
    //AuthBearer(token): AuthBearer
) -> HandlerResult {
    let query = query.0;
    daemon
        .path(query)
        .await
        .map(|r| {
            response::Json(json!({
                "status": "success",
                "data": r}))
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn plot_handler(
    Extension(daemon): Extension<Arc<DaemonHandle>>,
    //Extension(env_token): Extension<String>,
    query: Query<daemon_handle::PlotParams>,
    //AuthBearer(token): AuthBearer
) -> impl IntoResponse {
    //if token != env_token {
    //    return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    //}
    let query = query.0;
    let png = daemon
        .plot(query)
        .await.unwrap();
    
    let headers = [
                          (header::CONTENT_TYPE, "image/png"),
    ];

    (headers, png)
}

pub async fn h3plot_handler(
    Extension(daemon): Extension<Arc<DaemonHandle>>,
    //Extension(env_token): Extension<String>,
    query: Query<daemon_handle::H3PlotParams>,
    //AuthBearer(token): AuthBearer
) -> HandlerResult {
    let query = query.0;
    daemon
        .h3plot(query)
        .await
        .map(|r| {
            response::Json(json!({
                "status": "success",
                "data": r}))
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
