pub mod arena;
pub mod error;
pub mod mongo;
pub mod opt;
pub mod redis;
pub mod repo;

use crate::error::Error;
use arena::{ArenaFull, ArenaId, ClientData, UserId};
use axum::{
    extract::{Extension, Path, Query},
    routing::get,
    AddExtensionLayer, Json, Router,
};
use clap::Parser;
use opt::Opt;
use repo::Repo;
use serde::Deserialize;
use std::sync::Arc;
use pretty_env_logger;
use log::debug;//, info};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let opt = Opt::parse();
    debug!("{:?}", opt);

    let repo = Arc::new(Repo::new(opt.clone()));
    // let mongo = mongo::Mongo::new(opt.clone());
    // let redis = redis::Redis::new(opt.clone()).await.unwrap();
    redis::subscribe(opt.clone()).unwrap();

    let app = Router::new()
        .route("/", get(root))
        .route("/:id", get(arena))
        .layer(AddExtensionLayer::new(opt.clone()))
        .layer(AddExtensionLayer::new(repo));

    let app = if opt.nocors {
        app
    } else {
        app.layer(
            tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                axum::http::HeaderValue::from_static("*"),
            ),
        )
    };

    axum::Server::bind(&opt.bind)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryParams {
    user_id: Option<UserId>,
}

async fn arena(
    Path(id): Path<ArenaId>,
    Query(query): Query<QueryParams>,
    Extension(repo): Extension<Arc<Repo>>,
) -> Result<Json<ClientData>, Error> {
    let full: ArenaFull = repo.get_arena(id).await?;
    Ok(Json(ClientData::new(full, query.user_id)))
}

async fn root() -> &'static str {
    "lilarena"
}
