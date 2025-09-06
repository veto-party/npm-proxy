use std::{collections::HashMap, path::PathBuf, pin::Pin, sync::Arc};

use base64::prelude::{BASE64_STANDARD, Engine};
use reqwest::Url;
use serde_json::Value;
use tokio::{fs::{File, OpenOptions}, io::{AsyncReadExt, AsyncWriteExt}, sync::RwLock};

use crate::http::api::{error::Error, storage::ApiStorage};


pub struct ApiInner {
    pub registry_uri: String,

    pub resulting_registry_uri: String,

    pub cache: PathBuf,
}

pub struct ApiInnerResult {
    result: Arc<RwLock<Option<Result<ApiStorage, Error>>>>,
    fnc: Pin<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<ApiStorage, Error>> + Send + Sync>> + Send + Sync>>
}

impl ApiInnerResult {
    pub async  fn call(&self) -> Result<ApiStorage, Error> {
        if let Some(result) = self.result.read().await.as_ref() {
            return result.clone();
        }

        let mut writer = self.result.write().await;
        let result = (self.fnc)().await;
        let _ = writer.insert(result.clone());
        return result;
    }
}


impl Clone for ApiInner {
    fn clone(&self) -> Self {
        return ApiInner {
            cache: self.cache.clone(),
            registry_uri: self.registry_uri.clone(),
            resulting_registry_uri: self.resulting_registry_uri.clone()
        }
    }
}


impl ApiInner {

    fn modified(registry_uri: String, resulting_registry_uri: String, data: Vec<u8>) -> Vec<u8> {
        let mut result: serde_json::Value = serde_json::from_slice(&data).unwrap();
        let value = &mut result;

        {
            let mut stack = vec![value];

            while let Some(current) = stack.pop() {
                match current {
                    Value::String(s) => {
                        *s = s.replace(&registry_uri, &resulting_registry_uri);
                    }
                    Value::Array(arr) => {
                        for v in arr {
                            stack.push(v);
                        }
                    }
                    Value::Object(map) => {
                        for v in map.values_mut() {
                            stack.push(v);
                        }
                    }
                    _ => {}
                }
            }
        }

        return serde_json::to_vec(&result).unwrap();
    }

    pub fn do_load(self, uri: String) -> ApiInnerResult  {


        let outer_self_registry = self.registry_uri.clone();
        let outer_self_registry_result = self.resulting_registry_uri.clone();

        let func: Pin<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<ApiStorage, Error>> + Send + Sync>> + Send + Sync>> = Box::pin(move || {
            let uri_clone = uri.clone();

            let me = self.clone();

            let self_registry = outer_self_registry.clone();
            let self_registry_result = outer_self_registry_result.clone();


            let raw = Box::pin(async move || {
                if let Ok(val) = me.do_load_cache(&uri_clone).await {
                    return Ok(val);
                }


                println!("{}", uri_clone.clone());

                let mut url = Url::parse(&me.registry_uri.clone()).unwrap();
                url.set_path(&uri_clone);
                let response = reqwest::Client::new().get(url).send().await;

                if let Ok(val) = response {
                    let status = val.status();
                    let headers = val.headers().clone();

                    let mut headers_stored: HashMap<String, Vec<u8>> =  HashMap::new();

                    for (given_key, value) in headers.iter() {
                        if given_key.as_str().starts_with("content-") {
                            continue;
                        }

                        if given_key.as_str().contains("cookie") {
                            continue;
                        }


                        headers_stored.insert(given_key.to_string().clone(), value.as_bytes().to_vec());
                    }

                    let mut body = val.bytes().await.unwrap().to_vec();

                    if headers.contains_key("content-type") {
                        let given_type = headers.get("content-type").unwrap();
                        if given_type.to_str().unwrap().contains("json") {
                            body = ApiInner::modified(self_registry.clone(), self_registry_result.clone(), body);
                        }
                        headers_stored.insert("content-type".to_string(), given_type.as_bytes().to_vec());
                    }

                    if headers.contains_key("content-disposition") {
                        let given_type = headers.get("content-disposition").unwrap();
                        headers_stored.insert("content-disposition".to_string(), given_type.as_bytes().to_vec());
                    }

                    if headers.contains_key("content-transfer-encoding") {
                        let given_type = headers.get("content-transfer-encoding").unwrap();
                        headers_stored.insert("content-transfer-encoding".to_string(), given_type.as_bytes().to_vec());
                    }

                    headers_stored.remove("accept-ranges");
                    headers_stored.remove("server");
                    headers_stored.remove("connection");
                    headers_stored.remove("vary");

                    let stored = ApiStorage {
                        body: body,
                        headers: headers_stored
                    };

                    me.try_cache(status, uri_clone.clone(), &stored).await;
                    return Ok(stored);
                }

                if let Err(val) = response {
                    println!("{}", val.to_string());
                }

                return Err(Error::Unknown());
            });

            Box::pin(async move {
                let response = raw().await;
                return response.clone();
            })
        });

        return ApiInnerResult { result: Arc::new(RwLock::new(None)), fnc: func };
    }

    async fn try_cache(&self, response: reqwest::StatusCode, uri: String, stored: &ApiStorage) {
        if response.is_success() { 
            self.do_cache(uri, stored).await;
        }
    }

    async fn get_file_handle(&self, uri: String, options: &OpenOptions) -> Result<File, std::io::Error> {
        let mut path = self.cache.clone();
        path.push(BASE64_STANDARD.encode(uri) + ".bin");
        return options.open(path).await;
    }

    async fn do_cache(&self, uri: String, stored: &ApiStorage) {
        let result = serde_binary::to_vec(stored, serde_binary::binary_stream::Endian::Little).unwrap();
        let options = OpenOptions::new().create(true).append(false).write(true).create_new(true).clone();
        let me = self.clone();
        tokio::spawn(async move {
            me.get_file_handle(uri, &options).await.unwrap().write_all(&result).await.unwrap();
        });
    }

    async fn do_load_cache(&self, uri: &String) -> Result<ApiStorage, ()> {
        let file_handle = self.get_file_handle(uri.clone(), OpenOptions::new().create(false).append(false).write(false).create_new(false).read(true)).await;

        if file_handle.is_err() {
            return Err(());
        }

        let mut value = vec![];
        file_handle.unwrap().read_to_end(&mut value).await.unwrap();
        let result: ApiStorage = serde_binary::from_vec(value, serde_binary::binary_stream::Endian::Little).unwrap();
        return Ok(result);
    }

}

