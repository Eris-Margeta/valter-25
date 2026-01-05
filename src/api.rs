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
use crate::fs_writer::FsWriter; // NOVO
use std::sync::Arc;
use tracing::{info, error};
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
        if !state.config.clouds.iter().any(|c| c.name == name) {
            return Json(vec![]);
        }
        match state.cloud.fetch_all_dynamic(&name) {
            Ok(data) => Json(data),
            Err(_) => Json(vec![]),
        }
    }

    async fn island_data(&self, ctx: &Context<'_>, name: String) -> Json<Vec<Value>> {
        let state = ctx.data::<ApiState>().expect("ApiState missing");
        if !state.config.islands.iter().any(|i| i.name == name) {
            return Json(vec![]);
        }
        match state.cloud.fetch_all_dynamic(&name) {
            Ok(data) => Json(data),
            Err(_) => Json(vec![]),
        }
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
        let prompt = format!("Role: Strata Oracle for {}.\nData:\n{}\nUser Query: {}", state.config.global.company_name, context_str, question);
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
    /// Ažurira polje u Islandu (Projektu).
    /// Primjer: updateIslandField("Projekt", "Project Phoenix", "status", "Done")
    async fn update_island_field(
        &self, 
        ctx: &Context<'_>, 
        island_type: String, // npr. "Projekt"
        island_name: String, // npr. "Project Phoenix"
        key: String, 
        value: String
    ) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");

        // 1. Nađi putanju projekta u Bazi
        // Moramo dohvatiti 'path' kolonu iz tablice
        let rows = match state.cloud.fetch_all_dynamic(&island_type) {
            Ok(r) => r,
            Err(e) => return format!("DB Error: {}", e),
        };

        // Nađi red gdje je name == island_name
        let target_row = rows.iter().find(|r| {
            r.get("name").and_then(|v| v.as_str()) == Some(&island_name)
        });

        if let Some(row) = target_row {
            if let Some(path_str) = row.get("path").and_then(|v| v.as_str()) {
                let meta_path = Path::new(path_str).join("meta.yaml");
                
                // 2. Piši u fajl
                match FsWriter::update_yaml_field(&meta_path, &key, &value) {
                    Ok(_) => return "Success".to_string(), // Watcher će odraditi sync s bazom
                    Err(e) => return format!("Write Error: {}", e),
                }
            }
        }

        "Project not found".to_string()
    }

    /// Kreira novi Island (Projekt)
    async fn create_island(
        &self,
        ctx: &Context<'_>,
        island_type: String, // npr. "Projekt"
        name: String,
        initial_data: String // JSON string s početnim podacima: '{"klijent": "X", "operater": "Y"}'
    ) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState missing");

        // Nađi root path za taj tip islanda iz configa
        let island_def = state.config.islands.iter().find(|i| i.name == island_type);
        
        if let Some(def) = island_def {
            // Parsiraj JSON initial_data u Vec<(String, String)>
            let parsed_data: std::collections::HashMap<String, String> = 
                serde_json::from_str(&initial_data).unwrap_or_default();
            
            let data_vec: Vec<(String, String)> = parsed_data.into_iter().collect();

            // Ukloni trailing wildcard iz root_path (npr "./DEV/*" -> "./DEV")
            let root_clean = def.root_path.replace("/*", "");
            
            match FsWriter::create_island(&root_clean, &name, data_vec) {
                Ok(_) => "Created".to_string(),
                Err(e) => format!("Error: {}", e),
            }
        } else {
            "Unknown Island Type".to_string()
        }
    }
}

pub type StrataSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

async fn graphql_handler(schema: Extension<StrataSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")))
}

pub async fn start_server(cloud: Arc<SqliteManager>) -> anyhow::Result<()> {
    let config = Arc::new(Config::load("strata.config")?);

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ApiState { cloud, config })
        .finish();

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors);

    info!("GraphiQL IDE: http://localhost:8000/graphql");
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

