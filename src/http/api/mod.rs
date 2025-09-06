use std::{collections::HashMap, path, sync::Arc};

use axum::{extract::{Path, State}, routing::get, Router};
use tokio::sync::Mutex;

use crate::http::api::{api::Api, inner::ApiInner};

mod api;
mod inner;
mod error;
mod storage;


#[derive(Clone)]
struct ApiState {
    api: Arc<Mutex<Api>>
}

pub fn api_routes(router: Router) -> Router {

    let cache = path::absolute("./cache/".to_string()).unwrap();


    let api = Api {
        api_inner: Box::new(ApiInner { 
            registry_uri: "https://registry.npmjs.org/".to_string(),
            resulting_registry_uri: "http://localhost:5000/".to_string(),
            cache: cache
        }),
        // stored_responses: Arc::new(RwLock::new(HashMap::new())),
        running_requests: Arc::new(Mutex::new(HashMap::new()))
    };

    let api_state = ApiState { 
        api: Arc::new(Mutex::new(api))
    };


    router
        .route("/-/package/{package_name}/dist-tags", get(   
            |Path(package_name): Path<String>, State(api): State<ApiState>| async move {
                api.api.lock().await.get_dist_tags(package_name).await
            }
        ).with_state(api_state.clone()))
        .route("/{package_name}/-/{file_name}", get(
            |Path((package_name, file_name)): Path<(String, String)>, State(api): State<ApiState>| async move {
                api.api.lock().await.get_file(package_name, file_name).await
            }
        ).with_state(api_state.clone()))
        .route("/{package_name}", get(
            |Path(package_name): Path<String>, State(api): State<ApiState>| async move {
                api.api.lock().await.get_package_metadata(package_name).await
            }
        ).with_state(api_state.clone()))
}