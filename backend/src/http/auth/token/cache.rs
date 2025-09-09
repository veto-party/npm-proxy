use std::{collections::HashMap, sync::Arc, time::Instant};

use chrono::{Duration};
use redis::Commands;
use tokio::{sync::RwLock};
use tokio_schedule::Job;



#[derive(Clone)]
pub struct TokenCache {
    redis: redis::Client,
    cached: Arc<RwLock<HashMap<String, Instant>>>,
    cache_duration: Duration
}

impl TokenCache {

    pub async fn new(redis: redis::Client, cache_duration: Duration) -> Self {
        let element = Self{
            cache_duration,
            cached: Arc::new(RwLock::new(HashMap::new())),
            redis
        };

        let references = &element;

        tokio_schedule::every(5).second().perform(async move || {
            references.cleanup().await;
        }).await;

        return element;
    }

    pub async fn get_token_for_user(&self, token_to_check: String) -> bool {
        let mut exists = self.cached.read().await.contains_key(&token_to_check);

        if !exists {
            exists = self.redis.get_connection().unwrap().get("token.".to_string() + &token_to_check).unwrap_or(false);
            if exists {
                self.cached.write().await.insert(token_to_check, Instant::now());
            }
        }

        return exists;
    }

    pub async fn store_token_for_user(&self, token_to_check: String) {
        let () = self.redis.get_connection().unwrap().set("token.".to_string() + &token_to_check, true).unwrap();
        self.cached.write().await.insert(token_to_check, Instant::now());
    }

    pub async fn cleanup(&self) {
        let cache_duration = self.cache_duration.to_std().unwrap();
        let mut map =self.cached.write().await;
        let to_remove = map.clone().into_iter().filter(|(_, k)| {
            return k.elapsed() > cache_duration
        });

        for entry in to_remove {
            map.remove(&entry.0);
        }
    }
}