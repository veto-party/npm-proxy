use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::http::api::{inner::{ApiInner, ApiInnerResult}, storage::ApiStorage};


pub struct Api {
    pub running_requests: Arc<RwLock<HashMap<String, ApiInnerResult>>>,
    pub api_inner: Box<ApiInner>,
}

impl Clone for Api {
    fn clone(&self) -> Self {
        return Api {
            running_requests: self.running_requests.clone(),
            api_inner: self.api_inner.clone()
        }
    }
}

impl Api {
    async fn load(&mut self, uri: String) -> Result<ApiStorage, ()> {

        let mut created = false;
        let has_key = self.running_requests.read().await.contains_key(&uri);
        if !has_key {
            self.running_requests.write().await.insert(uri.clone(),self.api_inner.clone().do_load(uri.clone()));
            created = true;
        }

        let result = async {

            let running = self.running_requests.read().await;
            let option = running.get(&uri).unwrap();
            let result = option.call().await;
            
            if let Ok(awiated) = result {
                return Ok(awiated.clone());
            }

            return Err(());
        }.await;

        if created {
            self.running_requests.write().await.remove(&uri);
        }

        return result;
    }

    pub async fn get_package_metadata(&mut self, package_name: String) -> Result<ApiStorage, ()> {
        return Ok(self.load( urlencoding::encode(&package_name).to_string()).await.unwrap().clone());
    }

    pub async fn get_file(&mut self, package_name: String, file_name: String) -> Result<ApiStorage, ()> {
        return Ok(self.load( urlencoding::encode(&package_name).to_string() +  "/-/" + &urlencoding::encode(&file_name).to_string()).await.unwrap().clone());
    }

    pub async fn get_dist_tags(&mut self, package_name: String) -> Result<ApiStorage, ()> {
        return Ok(self.load( "-/package/".to_string() + &urlencoding::encode(&package_name).to_string() + "/dist-tags").await.unwrap().clone());
    }
}