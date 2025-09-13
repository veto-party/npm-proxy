use std::{collections::HashMap, sync::Arc};
use base64::{prelude::BASE64_STANDARD, Engine};
use tokio::{fs, sync::RwLock};

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

    pub async fn get_cached_packages(&self) -> Vec<String> {
        let result = fs::read_dir(self.api_inner.cache.clone()).await;

        if result.is_err() {
            return Vec::new();
        }

        let mut dir =result.unwrap();

        let mut vec: Vec<String> = Vec::new();

        while let Some(file) = dir.next_entry().await.unwrap() {
            if file.file_type().await.unwrap().is_dir() {
                continue;
            }

            if let Ok(result) = BASE64_STANDARD.decode(file.file_name().to_str().unwrap().to_string().strip_suffix(".bin").unwrap().to_string()) {
                vec.push(String::from_utf8(result).unwrap());
            }
        }

        return vec;
    }

    pub async fn delete_cached_file(&self, package_name: String) {
        let mut path = self.api_inner.cache.clone();
        path.push(BASE64_STANDARD.encode(urlencoding::encode(&package_name).to_string()) + ".bin");

        fs::remove_file(path).await.unwrap();
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