use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject};
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
use std::sync::Arc;
use tracing::{info, error};
use std::fs;
use std::path::Path;

pub struct ApiState {
    pub cloud: Arc<SqliteManager>,
}

#[derive(SimpleObject)]
pub struct Entity {
    pub id: String,
    pub name: String,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> &str {
        "Hello from Strata Oracle!"
    }

    async fn clients(&self, ctx: &Context<'_>) -> Vec<Entity> {
        let state = ctx.data::<ApiState>().expect("ApiState not found");
        match state.cloud.get_all("Client") {
            Ok(rows) => rows.into_iter().map(|(id, name)| Entity { id, name }).collect(),
            Err(e) => {
                error!("Failed to fetch clients: {}", e);
                vec![]
            }
        }
    }

    async fn operators(&self, ctx: &Context<'_>) -> Vec<Entity> {
        let state = ctx.data::<ApiState>().expect("ApiState not found");
        match state.cloud.get_all("Operator") {
            Ok(rows) => rows.into_iter().map(|(id, name)| Entity { id, name }).collect(),
            Err(e) => {
                error!("Failed to fetch operators: {}", e);
                vec![]
            }
        }
    }

    async fn projects(&self, ctx: &Context<'_>) -> Vec<Entity> {
        let state = ctx.data::<ApiState>().expect("ApiState not found");
        match state.cloud.get_all("Project") {
            Ok(rows) => rows.into_iter().map(|(id, name)| Entity { id, name }).collect(),
            Err(e) => {
                error!("Failed to fetch projects: {}", e);
                vec![]
            }
        }
    }

    async fn ask_oracle(&self, ctx: &Context<'_>, question: String) -> String {
        let state = ctx.data::<ApiState>().expect("ApiState not found");
        
        // Gather Context
        let clients = state.cloud.get_all("Client").unwrap_or_default();
        let operators = state.cloud.get_all("Operator").unwrap_or_default();
        let projects = state.cloud.get_all("Project").unwrap_or_default();
        
        let context_str = format!(
            "Known Projects: {:?}\nKnown Clients: {:?}\nKnown Operators: {:?}", 
            projects.iter().map(|(_, name)| name).collect::<Vec<_>>(),
            clients.iter().map(|(_, name)| name).collect::<Vec<_>>(),
            operators.iter().map(|(_, name)| name).collect::<Vec<_>>()
        );

        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            return "Error: GEMINI_API_KEY not set in environment.".to_string();
        }

        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
        );

        let prompt = format!(
            "System: You are the Strata Oracle, an AI managing a business database.\nContext: {}
User Question: {}",
            context_str, question
        );

        let body = serde_json::json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        match client.post(&url).json(&body).send().await {
            Ok(res) => {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    // Extract text from Gemini response structure
                    if let Some(text) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                        return text.to_string();
                    }
                }
                "Error parsing Gemini response.".to_string()
            }
            Err(e) => format!("Error calling Gemini: {}", e)
        }
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_project(&self, name: String, client: String, operator: String) -> String {
        // 1. Sanitize name
        let safe_name = name.replace(" ", "_");
        let path_str = format!("DEV/{}", safe_name);
        let path = Path::new(&path_str);

        if path.exists() {
            return format!("Error: Project '{}' already exists.", safe_name);
        }

        // 2. Create Directory
        if let Err(e) = fs::create_dir_all(path) {
            return format!("Error creating directory: {}", e);
        }

        // 3. Create meta.yaml
        let content = format!(
            "name: \"{}\"\nclient: \"{}\"\noperator: \"{}\"\ncreated_at: \"{}\"\nstatus: \"New\"\n",
            name, client, operator, chrono::Local::now().format("%Y-%m-%d")
        );

        let meta_path = path.join("meta.yaml");
        if let Err(e) = fs::write(meta_path, content) {
            return format!("Error writing meta.yaml: {}", e);
        }

        // 4. Return success (Watcher will pick it up)
        format!("Project '{}' created successfully. Watcher will ingest it shortly.", name)
    }
}

pub type StrataSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

async fn graphql_handler(
    schema: Extension<StrataSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")))
}

pub async fn start_server(cloud: Arc<SqliteManager>) -> anyhow::Result<()> {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ApiState { cloud })
        .finish();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors);

    info!("GraphiQL IDE: http://localhost:8000/graphql");

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
