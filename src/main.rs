
mod http;

use axum::{
    middleware, Router
};

use crate::http::api::api_routes;
use crate::http::auth::auth_middleware;

#[tokio::main]
async fn main() {
    
    let app = api_routes(Router::new())
        .route_layer(middleware::from_fn(auth_middleware));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}



