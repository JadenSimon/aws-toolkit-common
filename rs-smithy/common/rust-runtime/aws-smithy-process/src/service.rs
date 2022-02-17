/// Process abstraction to be used with `tower`
/// 
/// 

use tokio::process::{Child, Command};
use tokio::io::{Result};
use tokio::sync::Mutex;
use std::borrow::{BorrowMut, Borrow};
use std::collections::HashMap;
use std::ops::{Add, Deref};
use std::cell::{RefCell, Ref};
use std::process::Output;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};

type Handle = tokio::task::JoinHandle<Result<()>>;

use crate::request;

#[derive(Debug)]
pub struct ProcessError {
    code: i32,
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for ProcessError {
}

pub struct StartProcessRequest {
    pub command: String,
    pub arguments: Vec<String>,
    pub environment: Option<HashMap<String, String>>,
    pub working_directory: Option<std::path::PathBuf>,
}

impl StartProcessRequest {
    // shouldn't be public
    pub fn execute(self) -> Result<Child> {
        // TODO: working dir
        let vars = self.environment.unwrap_or(std::env::vars().into_iter().collect());

        let child = Command::new(&self.command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .envs(vars)
            .args(&self.arguments)
            .spawn()
            .expect(&format!("Failed to spawn {:?} -> {:?}", &self.command, &self.arguments)); // This panics...
    
        Ok(child)
    }
}

impl std::convert::From<tokio::io::Error> for ProcessError {
    fn from(_: tokio::io::Error) -> Self {
        Self { code: -1 }
    }
}


