use async_graphql::{Context, EmptySubscription, Object, Schema, Json};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tokio::net::TcpListener;
use crate::cloud::SqliteManager;
use crate::config::Config;
use crate::fs_writer::FsWriter;
use std::sync::Arc;
use tracing::{info};
use serde_json::Value;
use std::path::Path;

pub struct ApiState {
    pub cloud: Arc<SqliteManager>,
    pub config: Arc<Config>,
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
        if !state.config.clouds.iter().any(|c| c.name == name) { return Json(vec![]); }
        state.cloud.fetch_all_dynamic(&name).map(Json).unwrap_or(Json(vec![]))
    }

    async fn island_data(&self, ctx: &Context<'_>, name: String) -> Json<Vec<Value>> {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        if !state.config.islands.iter().any(|i| i.name == name) { return Json(vec![]); }
        state.cloud.fetch_all_dynamic(&name).map(Json).unwrap_or(Json(vec![]))
    }

    async fn pending_actions(&self, ctx: &Context<'_>) -> Json<Vec<Value>> {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        state.cloud.fetch_pending_actions().map(Json).unwrap_or(Json(vec![]))
    }

    async fn ask_oracle(&self, ctx: &Context<'_>, question: String) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        
        let mut context_str = String::from("System Context:\n");
        for cloud_def in &state.config.clouds {
             if let Ok(data) = state.cloud.fetch_all_dynamic(&cloud_def.name) {
                 let preview: Vec<_> = data.iter().take(20).collect();
                 context_str.push_str(&format!("Table '{}': {:?}\n", cloud_def.name, preview));
             }
        }
        for island_def in &state.config.islands {
             if let Ok(data) = state.cloud.fetch_all_dynamic(&island_def.name) {
                 let preview: Vec<_> = data.iter().take(20).collect();
                 context_str.push_str(&format!("Projects '{}': {:?}\n", island_def.name, preview));
             }
        }

        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        if api_key.is_empty() { return "Error: GEMINI_API_KEY missing.".to_string(); }

        let client = reqwest::Client::new();
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key);
        let prompt = format!("Role: Valter Oracle for {}.\nData:\n{}\nUser Query: {}", state.config.global.company_name, context_str, question);
        let body = serde_json::json!({ "contents": [{ "parts": [{"text": prompt}] }] });

        match client.post(&url).json(&body).send().await {
            Ok(res) => {
                if let Ok(json) = res.json::<Value>().await {
                    if let Some(text) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                        return text.to_string();
                    }
                }
                "AI Parsing Error".to_string()
            }
            Err(e) => format!("AI Network Error: {}", e)
        }
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn update_island_field(&self, ctx: &Context<'_>, island_type: String, island_name: String, key: String, value: String) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        if let Ok(rows) = state.cloud.fetch_all_dynamic(&island_type) {
            if let Some(row) = rows.iter().find(|r| r.get("name").and_then(|v| v.as_str()) == Some(&island_name)) {
                if let Some(path_str) = row.get("path").and_then(|v| v.as_str()) {
                    let meta_path = Path::new(path_str).join("meta.yaml");
                    if FsWriter::update_yaml_field(&meta_path, &key, &value).is_ok() {
                        return "Success".to_string();
                    }
                }
            }
        }
        "Error updating field".to_string()
    }

    async fn create_island(&self, ctx: &Context<'_>, island_type: String, name: String, initial_data: String) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        if let Some(def) = state.config.islands.iter().find(|i| i.name == island_type) {
             let parsed: std::collections::HashMap<String, String> = serde_json::from_str(&initial_data).unwrap_or_default();
             let data: Vec<_> = parsed.into_iter().collect();
             let root = def.root_path.replace("/*", "");
             if FsWriter::create_island(&root, &name, data).is_ok() { return "Created".to_string(); }
        }
        "Error creating island".to_string()
    }

    async fn resolve_action(&self, ctx: &Context<'_>, action_id: String, choice: String) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        match choice.as_str() {
            "APPROVE" => {
                match state.cloud.approve_pending_creation(&action_id) {
                    Ok(id) => format!("Created Entity: {}", id),
                    Err(e) => format!("Error approving: {}", e)
                }
            },
            "REJECT" => {
                let _ = state.cloud.reject_pending_action(&action_id);
                "Rejected".to_string()
            },
            _ => "Unknown choice".to_string()
        }
    }
}

pub type ValterSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

async fn graphql_handler(schema: Extension<ValterSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")))
}

pub async fn start_server(cloud: Arc<SqliteManager>, config: Arc<Config>) -> anyhow::Result<()> {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ApiState { cloud, config: config.clone() })
        .finish();

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors);

    // NOVO: Čitamo port iz configa
    let port = config.global.port;
    let addr = format!("0.0.0.0:{}", port);
    
    info!("GraphiQL IDE: http://localhost:{}/graphql", port);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

