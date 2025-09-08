use std::collections::HashMap;

use axum::{body::Body, extract::Query, http::{header::RETRY_AFTER, Response, StatusCode}, response::{AppendHeaders, IntoResponse, Redirect}, routing::{get, post}, Json, Router};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use redis::{Commands};
use serde::{Deserialize, Serialize};
use serde_json::{json};

use crate::http::auth::authenticator::Authenticator;

#[derive(Clone)]
pub struct AuthenticatorApi {
    self_url: String,
    redis: redis::Client,
    authenticator: Authenticator,
}

#[derive(Deserialize, Serialize, redis_macros::FromRedisValue, redis_macros::ToRedisArgs)]
enum AuthenticatorStatus {
    Empty(),
    Stored(String),
    Unknown()
}

impl AuthenticatorApi {

    pub fn new(self_url: String, redis: redis::Client, authenticator: Authenticator) -> Self {
        return Self {
            authenticator,
            redis,
            self_url
        }
    }

    pub async fn unlock(&self, id: String, token: String) -> Result<(), ()> {
        if let Ok(mut connection) = self.redis.get_connection() {
            let some: AuthenticatorStatus = connection.get(&id).unwrap();
            if let AuthenticatorStatus::Stored(_) = some {
                return Err(());
            }
            
            let () = connection.set(&id, AuthenticatorStatus::Stored(token)).unwrap();
            return Ok(());
        }

        return Err(());
    }

    pub fn routes(&self, router: Router) -> Router {

        let mut resulting_router = router;

        {
            let client = self.redis.clone();

            #[derive(Serialize, Deserialize, Clone)]
            struct LoginResponse {
                loginUrl: String,
                doneUrl: String
            }

            let base = self.self_url.clone();

            resulting_router = resulting_router.route("/-/v1/login",  post(async move || {

                let uuid = uuid::Uuid::new_v4();

                if let Ok(mut connection) = client.get_connection() {
                    let response: Result<AuthenticatorStatus, redis::RedisError> =connection.get(&uuid.to_string());
                    if let Err(_) = response {

                        let () = connection.set(&uuid.to_string(), AuthenticatorStatus::Empty()).unwrap();
                        
                        let response = LoginResponse {
                            loginUrl: (base.clone() + "login?id=" + &urlencoding::encode(&uuid.to_string()).to_string()).to_string(),
                            doneUrl: (base.clone() + "check_done?id=" + &urlencoding::encode(&uuid.to_string()).to_string()).to_string(),
                        };

                        return Ok(Json(json!(response.clone())));
                    }
                }

                return Err(());
            }));
        }

        {
            let auth = self.authenticator.clone();
            resulting_router = resulting_router.route("/login", get(async move |Query(all): Query<HashMap<String, String>>, jar: CookieJar| {
                let id = all.get("id").unwrap().clone();
                
                let (token, (uri, _, _)) = auth.get_redirect_url(id);

                return (
                    jar.add(Cookie::new("_csrf", token.secret().clone())),
                    Redirect::temporary(&uri.to_string())
                );
            }))
        }

        {
            let client = self.redis.clone();
            #[derive(Serialize, Deserialize, Clone)]
            struct TokenResponse {
                token: String,
            }

            resulting_router = resulting_router.route("/check_done", get(async move |Query(all): Query<HashMap<String, String>>| {

                let id = all.get("id").unwrap().clone();

                if let Ok(mut connection) = client.get_connection() {
                    let el: AuthenticatorStatus = connection.get(&id).unwrap();
                    match el {
                        AuthenticatorStatus::Unknown() => {
                            return Err(());
                        }
                        AuthenticatorStatus::Empty() => {
                            return Ok((
                                StatusCode::ACCEPTED,
                                AppendHeaders([
                                (RETRY_AFTER, "1")
                            ]),
                            Json("{}").into_response()));
                        },
                        AuthenticatorStatus::Stored(result) => {
                            let () = connection.del(&id).unwrap();
                            return Ok((
                                StatusCode::OK,
                                AppendHeaders([
                                    (RETRY_AFTER, "1")
                                ]),
                                Json(json!(TokenResponse {
                                token: result
                            })).into_response()));
                        }
                    }
                }

                return Err(());
            }))
        }

        return resulting_router;
    }
}