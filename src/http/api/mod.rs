use std::{collections::HashMap, env, path, sync::Arc};

use axum::{extract::{Path, State}, routing::get, Router};
use tokio::sync::{Mutex, RwLock};

use crate::http::api::{api::Api, inner::ApiInner};

mod api;
mod inner;
mod error;
mod storage;


#[derive(Clone)]
struct ApiState {
    api: Api
}

pub fn api_routes(router: Router) -> Router {

    let cache = path::absolute("./cache/".to_string()).unwrap();


    let api = Api {
        api_inner: Box::new(ApiInner { 
            registry_uri: env::var("PROXY_REGISTRY_URI").unwrap_or("https://registry.npmjs.org/".to_string()),
            resulting_registry_uri: env::var("PROXY_REGISTRY_HOST").unwrap_or("http://localhost:5000/".to_string()),
            cache: cache
        }),
        // stored_responses: Arc::new(RwLock::new(HashMap::new())),
        running_requests: Arc::new(RwLock::new(HashMap::new()))
    };

    let api_state = ApiState { 
        api: api
    };


    router
        .route("/-/package/{package_name}/dist-tags", get(   
            |Path(package_name): Path<String>, State(mut api): State<ApiState>| async move {
                api.api.get_dist_tags(package_name).await
            }
        ).with_state(api_state.clone()))
        .route("/{package_name}/-/{file_name}", get(
            |Path((package_name, file_name)): Path<(String, String)>, State(mut api): State<ApiState>| async move {
                api.api.get_file(package_name, file_name).await
        }).with_state(api_state.clone()))
        .route("/@{package_namespace}/{package_name}/-/{file_name}", get(
            |Path((package_namespace, package_name, file_name)): Path<(String, String, String)>, State(mut api): State<ApiState>| async move {
                api.api.get_file("@".to_string() + &package_namespace + "/" + &package_name, file_name).await
            }
        ).with_state(api_state.clone()))
        .route("/{package_name}", get(
            |Path(package_name): Path<String>, State(mut api): State<ApiState>| async move {
                api.api.get_package_metadata(package_name).await
            }
        ).with_state(api_state.clone()))
}