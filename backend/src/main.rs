
mod http;
mod config;

use std::clone;
use std::collections::HashMap;

use axum::body::Body;
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

    let app = api.routes(api_routes(Router::new(), &conf))
        .route("/", get(|State((auth, api)): State<(Authenticator, AuthenticatorApi)>, Query(params):Query<HashMap<String, String>>, jar: CookieJar| async move {
            if params.contains_key("code") {
                let token = auth.get_from_redirected(params.get("code").unwrap().clone(), jar.get("_csrf").unwrap().to_string()).await.secret().to_string();
                let result = Body::new(token.clone()).into_response();


                if params.contains_key("state") {
                    let _ = api.unlock(params.get("state").unwrap().to_string(), token).await;
                }

                return (
                    jar,
                    result
                );
            }

            let (token, (url, _, _)) = auth.get_redirect_url("".to_string());
            return (
                jar.add(Cookie::new("_csrf", token.secret().clone())), 
                Redirect::temporary(&url.as_str().to_string()).into_response()
            );
        }).with_state((auth.clone(), api.clone())))
        .route_layer(middleware::from_fn_with_state(auth.clone(), |State(state): State<Authenticator>, req: Request, next: Next| async move  {
            if req.uri().path().eq("/") ||  req.uri().path().eq("/-/v1/login") || req.uri().path().starts_with("/login") || req.uri().path().starts_with("/check_done") {
                let respsonse = next.run(req).await;
                return Ok(respsonse);
            }
            let result = state.middleware(req, next).await;            
            return result;
        })).route_layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}



