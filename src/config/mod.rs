use std::env;


pub struct Config {
    pub self_url: String,
    pub registry_url: String,
    pub oidc_url: String,
    pub oidc_client_secret: String,
    pub oidc_client_id: String,
    pub redis_uri: String,
}

impl Config {
    pub fn new() -> Self {
        return Self {
            self_url: env::var("PROXY_REGISTRY_HOST").unwrap_or("http://localhost:5000/".to_string()),
            registry_url: env::var("PROXY_REGISTRY_URI").unwrap_or("https://registry.npmjs.org/".to_string()),
            oidc_url:  env::var("OIDC_ISSUER_URL").unwrap_or("https://gitlab.git.veto.dev".to_string()),
            oidc_client_secret: env::var("OIDC_CLIENT_ID").unwrap_or("some-id".to_string()),
            oidc_client_id: env::var("OIDC_CLIENT_SECRET").unwrap_or("some-secret".to_string()),
            redis_uri: env::var("REDIS_URI").unwrap_or("redis://localhost:6379".to_string())
        }
    }
}