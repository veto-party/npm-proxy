use std::collections::HashMap;

use axum::{body::Body, http::Response, response::IntoResponse};
use serde::{Deserialize, Serialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct ApiStorage {
    pub headers: HashMap<String, Vec<u8>>,
    pub body: Vec<u8>,
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