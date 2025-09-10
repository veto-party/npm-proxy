use std::{sync::Arc, time::Duration};

use rand::{distr::Alphanumeric, rng, Rng};

use crate::{domain::Tokens::Tokens, http::auth::token::cache::TokenCache};


#[derive(Clone)]
pub struct TokenApi {
    cache: Arc<TokenCache>,
}

impl TokenApi {

    pub async fn new(redis: redis::Client, duration: Duration) -> Self {
        return Self { cache: TokenCache::new(redis, duration).await };
    }

    pub async fn create_token(&self, _token: Tokens) -> String {
        let mut token: String = rng()
        .sample_iter(&Alphanumeric)
        .take(14)
        .map(char::from)
        .collect();
        token.insert_str(0, "veto-np_");
        self.cache.store_token_for_user(token.clone()).await;
        return token;
    }

    pub async fn verify_token(&self, token: String) -> bool {
        return self.cache.get_token_for_user(token).await;
    }
}
