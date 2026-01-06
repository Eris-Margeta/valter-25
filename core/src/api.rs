// core/src/api.rs (ISPRAVLJENO)

use crate::cloud::SqliteManager;
use crate::config::Config;
use crate::fs_writer::FsWriter;
use crate::processor::EventProcessor; // Dodano za rescan
use async_graphql::{Context, EmptySubscription, Json, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::{header, Method, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(RustEmbed)]
#[folder = "../dashboard/dist"]
struct Assets;

pub struct ApiState {
    pub cloud: Arc<SqliteManager>,
    pub config: Arc<Config>,
    pub processor: Arc<EventProcessor>, // DODANO: Treba nam procesor za rescan
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn config(&self, ctx: &Context<'_>) -> Json<Config> {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        Json(state.config.as_ref().clone())
    }

    async fn cloud_data(&self, ctx: &Context<'_>, name: String) -> Json<Vec<Value>> {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        state
            .cloud
            .fetch_all_dynamic(&name)
            .map(Json)
            .unwrap_or(Json(vec![]))
    }

    async fn island_data(&self, ctx: &Context<'_>, name: String) -> Json<Vec<Value>> {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        state
            .cloud
            .fetch_all_dynamic(&name)
            .map(Json)
            .unwrap_or(Json(vec![]))
    }

    async fn pending_actions(&self, ctx: &Context<'_>) -> Json<Vec<Value>> {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        state
            .cloud
            .fetch_pending_actions()
            .map(Json)
            .unwrap_or(Json(vec![]))
    }

    async fn ask_oracle(&self, ctx: &Context<'_>, question: String) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        let mut context_str = String::from("System Context:\n");
        for cloud in &state.config.clouds {
            context_str.push_str(&format!("Table: {}\n", cloud.name));
        }

        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            return "Error: GEMINI_API_KEY missing.".to_string();
        }

        let client = reqwest::Client::new();
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key);
        let prompt = format!(
            "Role: Valter Oracle.\nContext: {}\nQuery: {}",
            context_str, question
        );
        let body = serde_json::json!({ "contents": [{ "parts": [{"text": prompt}] }] });

        match client.post(&url).json(&body).send().await {
            Ok(res) => res
                .json::<Value>()
                .await
                .ok()
                .and_then(|j| {
                    j["candidates"][0]["content"]["parts"][0]["text"]
                        .as_str()
                        .map(|s| s.to_string())
                })
                .unwrap_or("AI Parse Error".to_string()),
            Err(e) => format!("AI Error: {}", e),
        }
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // DODANO: rescanIslands mutacija
    async fn rescan_islands(&self, ctx: &Context<'_>) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        info!("Manual Rescan Requested via API");
        state.processor.scan_on_startup();
        "Rescan Complete".to_string()
    }

    async fn update_island_field(
        &self,
        ctx: &Context<'_>,
        island_type: String,
        island_name: String,
        key: String,
        value: String,
    ) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        if let Ok(rows) = state.cloud.fetch_all_dynamic(&island_type) {
            if let Some(row) = rows
                .iter()
                .find(|r| r.get("name").and_then(|v| v.as_str()) == Some(&island_name))
            {
                if let Some(path_str) = row.get("path").and_then(|v| v.as_str()) {
                    let meta_path = Path::new(path_str).join("meta.yaml");
                    if FsWriter::update_yaml_field(&meta_path, &key, &value).is_ok() {
                        return "Success".to_string();
                    }
                }
            }
        }
        "Error".to_string()
    }

    async fn create_island(
        &self,
        ctx: &Context<'_>,
        island_type: String,
        name: String,
        initial_data: String,
    ) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        if let Some(def) = state.config.islands.iter().find(|i| i.name == island_type) {
            let parsed: std::collections::HashMap<String, String> =
                serde_json::from_str(&initial_data).unwrap_or_default();
            let data: Vec<_> = parsed.into_iter().collect();
            let root = def.root_path.replace("/*", "");
            if FsWriter::create_island(&root, &name, data).is_ok() {
                return "Created".to_string();
            }
        }
        "Error".to_string()
    }

    async fn resolve_action(&self, ctx: &Context<'_>, action_id: String, choice: String) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        match choice.as_str() {
            "APPROVE" => state
                .cloud
                .approve_pending_creation(&action_id)
                .map(|id| format!("Created: {}", id))
                .unwrap_or("Error".to_string()),
            "REJECT" => {
                let _ = state.cloud.reject_pending_action(&action_id);
                "Rejected".to_string()
            }
            _ => "Unknown".to_string(),
        }
    }
}

pub type ValterSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

async fn graphql_handler(schema: Extension<ValterSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }
    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if let Some(content) = Assets::get("index.html") {
                let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}

pub async fn start_server(
    cloud: Arc<SqliteManager>,
    config: Arc<Config>,
    processor: Arc<EventProcessor>, // DODANO: Proslijedi procesor
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ApiState {
            cloud,
            config: config.clone(),
            processor,
        }) // DODANO: Stavi procesor u state
        .finish();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT]);

    let app = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .fallback(static_handler)
        .layer(Extension(schema))
        .layer(cors);

    let port = config.global.port;
    let addr = format!("0.0.0.0:{}", port);
    info!("🚀 API available at: http://localhost:{}", port);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_rx.recv().await.ok();
            info!("API shutting down for reload...");
        })
        .await?;
    Ok(())
}
