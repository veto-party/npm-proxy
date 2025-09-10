
mod http;
mod config;
mod domain;

use std::collections::HashMap;

use axum::extract::{Query, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{
    middleware, Router
};
use axum_extra::extract::CookieJar;
use chrono::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::http::api::api_routes;
use crate::http::auth::api::AuthenticatorApi;
use crate::http::auth::authenticator::Authenticator;

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let conf = config::Config::new();

    let redis = redis::Client::open(conf.redis_uri.clone()).unwrap();

    println!("Discovery of oidc");
    let auth = Authenticator::create(&conf, redis.clone(), Duration::minutes(2).to_std().unwrap()).await;

    let api = AuthenticatorApi::new(conf.self_url.clone(), redis.clone(), auth.clone());

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let frontend = ServeDir::new("./public/");

    let mut app = api.routes(api_routes(Router::new(), &conf))
        .route("/", get(|| async move {
            return Redirect::temporary("/ui/").into_response();
        }))
        .route_layer(middleware::from_fn_with_state((auth.clone(), api.clone()), |State((state, api)): State<(Authenticator, AuthenticatorApi)>, Query(params):Query<HashMap<String, String>>, jar: CookieJar, req: Request, next: Next| async move  {
            if req.uri().path().eq("/")  {
                let token = state.get_from_redirected(params.get("code").unwrap().clone(), jar.get("_csrf").unwrap().to_string()).await;
                if params.contains_key("state") {
                    let _ = api.unlock(params.get("state").unwrap().to_string(), token).await;
                }
            }

            if req.uri().path().eq("/-/v1/login") || req.uri().path().eq("/login") || req.uri().path().eq("/check_done") || req.uri().path().eq("/") {
                return Ok(next.run(req).await);
            }


            let result = state.middleware(req, next).await;            
            return result;
        }));

        if conf.dev {
            app = app.route_layer(cors);
        }        

        app = app.nest_service("/ui", frontend);

        println!("Starting app on port: 5000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}



