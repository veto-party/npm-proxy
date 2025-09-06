use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::task_local;

#[derive(Clone)]
struct CurrentUser {
    id: String,
}

task_local! {
    pub static USER: CurrentUser;
}

pub async fn auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    return Ok(next.run(req).await);
    // let auth_header = req
    //     .headers()
    //     .get(header::AUTHORIZATION)
    //     .and_then(|header| header.to_str().ok())
    //     .ok_or(StatusCode::UNAUTHORIZED)?;

    // if let Some(user) = authorize(auth_header).await {
    //     return Ok(USER.scope(user, next.run(req)).await)
    // }

    // Err(StatusCode::UNAUTHORIZED)
}

async fn authorize(token: &str) -> Option<CurrentUser> {
    Some(CurrentUser { id: "hello-world".to_owned() })
}