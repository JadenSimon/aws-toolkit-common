// Request + builder for process commands

use std::fmt;
use serde::{self, Deserialize, Serialize};
use std::option::{Option};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    /// Required
    #[serde(rename = "jsonrpc")] 
    pub version: String,
    /// Required
    pub method: String,
    /// Not required for notifications
    // TODO: there should be an 'operation' subset of `Request` that omits the ID since it needs to be unique
    pub id: Option<String>,
    /// Not required (TODO: support array)
    /// Also how to restrict to only struct/vec ?
    /// This could just be serde_json::Value::Object ?
    pub params: Option<Value>,
}


pub struct RequestBuilder {
    version: Option<String>,
    method: Option<String>,
    id: Option<String>,
    params: Option<Value>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self { version: None, method: None, id: None, params: None }
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }

    // TODO: don't use IO error
    pub fn build(self) -> std::result::Result<Request, std::io::Error> {
        if let None = self.version {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "No version"))
        } else if let None = self.method {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "No method"))
        } else {
            Ok(Request {
                version: self.version.unwrap(),
                method: self.method.unwrap(),
                id: self.id,
                params: self.params,
            })
        }
    }
}