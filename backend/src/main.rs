
mod http;
mod config;

use std::clone;
use std::collections::HashMap;

use axum::body::{self, Body};
use axum::extract::{Query, Request, State};
use axum::http::{HeaderValue, Response};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{
    middleware, Router
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::http::api::api_routes;
use crate::http::auth::api::{self, AuthenticatorApi};
use crate::http::auth::authenticator::Authenticator;

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let conf = config::Config::new();

    let auth = Authenticator::create(&conf).await;

    let redis = redis::Client::open(conf.redis_uri.clone()).unwrap();

    let api = AuthenticatorApi::new(conf.self_url.clone(), redis, auth.clone());

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let frontend = ServeDir::new("./public/");

    let app = api.routes(api_routes(Router::new(), &conf))
        .route("/", get(|| async move {
            return Redirect::temporary("/ui/").into_response();
        }))
        .route_layer(middleware::from_fn_with_state((auth.clone(), api.clone()), |State((state, api)): State<(Authenticator, AuthenticatorApi)>, Query(params):Query<HashMap<String, String>>, jar: CookieJar, req: Request, next: Next| async move  {
            if req.uri().path().eq("/")  {
                let token = state.get_from_redirected(params.get("code").unwrap().clone(), jar.get("_csrf").unwrap().to_string()).await.secret().to_string();
                if params.contains_key("state") {
                    let _ = api.unlock(params.get("state").unwrap().to_string(), token).await;
                }
            }

            if req.uri().path().eq("/-/v1/login") || req.uri().path().eq("/login") || req.uri().path().eq("/check_done") || req.uri().path().eq("/") {
                return Ok(next.run(req).await);
            }


            let result = state.middleware(req, next).await;            
            return result;
        })).route_layer(cors)
        .nest_service("/ui", frontend);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}



