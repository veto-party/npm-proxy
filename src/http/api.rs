use std::{collections::HashMap, fs::{File, OpenOptions}, io::{Read, Write}, path::{self, PathBuf}, pin::Pin, sync::Arc, task::Context};

use axum::{body::Body, extract::{Path, State}, http::Response, response::IntoResponse, routing::get, Router};
use base64::{prelude::BASE64_STANDARD, Engine};
use futures::{Future, FutureExt};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;


#[derive(Debug, Clone)]
enum Error {
    Api(),
    Unknown()
}


#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
struct ApiStorage {
    headers: HashMap<String, Vec<u8>>,
    body: Vec<u8>,
}


impl IntoResponse for ApiStorage {
    fn into_response(self) -> Response<Body> {
        let mut builder = Response::builder();

        for (key, value) in self.headers.into_iter() {
            builder = builder.header(key, value);   
        }
        
        builder = builder.header("content-length", self.body.len());

        return builder.body(Body::from(self.body)).unwrap();
    }
}


#[derive(Clone)]
struct ApiState {
    api: Arc<Mutex<Api>>
}

pub fn api_routes(router: Router) -> Router {

    let cache = path::absolute("./cache/".to_string()).unwrap();


    let api = Api {
        api_inner: Box::new(ApiInner { 
            registry_uri: "https://registry.npmjs.org/".to_string(),
            resulting_registry_uri: "http://localhost:5000/".to_string(),
            cache: cache,
            value: HashMap::new()
        }),
        // stored_responses: Arc::new(RwLock::new(HashMap::new())),
        running_requests: Arc::new(Mutex::new(HashMap::new()))
    };

    let api_state = ApiState { 
        api: Arc::new(Mutex::new(api))
    };


    router
        .route("/-/package/{package_name}/dist-tags", get(   
            |Path(package_name): Path<String>, State(api): State<ApiState>| async move {
                api.api.lock().await.get_dist_tags(package_name).await
            }
        ).with_state(api_state.clone()))
        .route("/{package_name}/-/{file_name}", get(
            |Path((package_name, file_name)): Path<(String, String)>, State(api): State<ApiState>| async move {
                api.api.lock().await.get_file(package_name, file_name).await
            }
        ).with_state(api_state.clone()))
        .route("/{package_name}", get(
            |Path(package_name): Path<String>, State(api): State<ApiState>| async move {
                api.api.lock().await.get_package_metadata(package_name).await
            }
        ).with_state(api_state.clone()))
}

struct Api {
    running_requests: Arc<Mutex<HashMap<String, Pin<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<ApiStorage, Error>> + Send + Sync>> + Send + Sync>>>>>,
    api_inner: Box<ApiInner>,
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
            
            if let Ok(awiated) = option().await {
                return Ok(awiated);
            }

            return Err(());
        }.await;

        if created {
            self.running_requests.lock().await.remove(&uri);
        }

        return result;
    }

    async fn get_package_metadata(&mut self, package_name: String) -> Result<ApiStorage, ()> {
        return Ok(self.load( package_name).await.unwrap().clone());
    }

    async fn get_file(&mut self, package_name: String, file_name: String) -> Result<ApiStorage, ()> {
        return Ok(self.load( package_name +  "/-/" + &file_name).await.unwrap().clone());
    }

    async fn get_dist_tags(&mut self, package_name: String) -> Result<ApiStorage, ()> {
        return Ok(self.load( "-/package/".to_string() + &package_name + "/dist-tags").await.unwrap().clone());
    }

}

struct ApiInner {
    /**
     * We expect this registry uri to end with /
     */
    registry_uri: String,

    resulting_registry_uri: String,

    cache: PathBuf,

    value: HashMap<String, ApiStorage>
}


impl Clone for ApiInner {
    fn clone(&self) -> Self {
        return ApiInner {
            cache: self.cache.clone(),
            registry_uri: self.registry_uri.clone(),
            resulting_registry_uri: self.resulting_registry_uri.clone(),
            value: HashMap::new()
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

    fn do_load(self, uri: String) -> Pin<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<ApiStorage, Error>> + Send + Sync>> + Send + Sync>>  {


        let outer_self_registry = self.registry_uri.clone();
        let outer_self_registry_result = self.resulting_registry_uri.clone();

        return Box::pin(move || {
            let uri_clone = uri.clone();

            let me = self.clone();

            let uri_clone2 = uri.clone();
            let me2 = self.clone();

            let uri_clone3 = uri_clone2.clone();
            let mut me3 = me2.clone();

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
                if let Some(cached) = me3.value.get(&uri_clone3) {
                return Ok(cached.clone());
            }

                let response = raw().await;
                if let Ok(result)  = response.clone() {
                    me3.value.insert(uri_clone3.clone(), result);
                }


                return response.clone();
            })
        });
    }

    async fn try_cache(&self, response: reqwest::StatusCode, uri: String, stored: &ApiStorage) {
        if response.is_success() { 
            self.do_cache(uri, stored).await;
        }
    }

    fn get_file_handle(&self, uri: String, options: &OpenOptions) -> Result<File, std::io::Error> {
        let mut path = self.cache.clone();
        path.push(BASE64_STANDARD.encode(uri) + ".bin");
        return options.open(path);
    }

    async fn do_cache(&self, uri: String, stored: &ApiStorage) {
        let result = serde_binary::to_vec(stored, serde_binary::binary_stream::Endian::Little).unwrap();
        self.get_file_handle(uri, OpenOptions::new().create(true).append(false).write(true).create_new(true)).unwrap().write_all(&result).unwrap();
    }

    async fn do_load_cache(&self, uri: &String) -> Result<ApiStorage, ()> {
        let file_handle = self.get_file_handle(uri.clone(), OpenOptions::new().create(false).append(false).write(false).create_new(false).read(true));

        if file_handle.is_err() {
            return Err(());
        }

        let mut value = vec![];
        file_handle.unwrap().read_to_end(&mut value).unwrap();
        let result: ApiStorage = serde_binary::from_vec(value, serde_binary::binary_stream::Endian::Little).unwrap();
        return Ok(result);
    }

}

