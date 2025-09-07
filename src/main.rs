
mod http;

use std::collections::HashMap;

use axum::body::Body;
use axum::extract::{Query, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{
    middleware, Router
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;

use crate::http::api::api_routes;
use crate::http::auth::authenticator::Authenticator;

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let auth = Authenticator::create().await;
    let app = api_routes(Router::new())
        .route("/", get(|State(state): State<Authenticator>, Query(params):Query<HashMap<String, String>>, jar: CookieJar| async move {
            if params.contains_key("code") {

                let result = Body::new(state.get_from_redirected(params.get("code").unwrap().clone(), jar.get("_csrf").unwrap().to_string()).await.secret().to_string()).into_response();
                return (
                    jar,
                    result
                );
            }


            let (url, token, _) = state.get_redirect_url();

            return (
                jar.add(Cookie::new("_csrf", token.secret().clone())), 
                Redirect::temporary(&url.as_str().to_string()).into_response()
            );
        }).with_state(auth.clone()))
        .route_layer(middleware::from_fn_with_state(auth.clone(), |State(state): State<Authenticator>, req: Request, next: Next| async move  {
            if req.uri().path().eq("/") {
                return Ok(next.run(req).await);
            }
            return state.middleware(req, next).await;
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}



