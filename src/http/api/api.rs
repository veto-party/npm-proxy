use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::http::api::{inner::{ApiInner, ApiInnerResult}, storage::ApiStorage};


pub struct Api {
    pub running_requests: Arc<Mutex<HashMap<String, ApiInnerResult>>>,
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
        if !self.running_requests.lock().await.contains_key(&uri) {
            self.running_requests.lock().await.insert(uri.clone(),self.api_inner.clone().do_load(uri.clone()));
            created = true;
        }

        let result = async {

            let running = self.running_requests.lock().await;
            let option = running.get(&uri).unwrap();
            
            if let Ok(awiated) = option.call().await {
                return Ok(awiated);
            }

            return Err(());
        }.await;

        if created {
            self.running_requests.lock().await.remove(&uri);
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