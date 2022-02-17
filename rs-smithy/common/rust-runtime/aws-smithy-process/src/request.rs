// Request + builder for process commands

use std::fmt;

#[derive(Debug)]
pub struct Request {
    /// List of strings
    pub arguments: Vec<String>,

    // TODO: environment variables, cwd, etc.
}


pub struct RequestBuilder {
    arguments: Vec<String>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self { 
            arguments: Vec::new() 
        }
    }

    pub fn add_flag(mut self, switch: impl Into<String>) -> Self {
        self.arguments.push(switch.into());
        self
    }

    pub fn add_option<T: fmt::Display>(&mut self, switch: String, value: T) {
        self.arguments.push(switch);
        self.arguments.push(value.to_string());
    }

    pub fn add_command(mut self, command: impl Into<String>) -> Self {
        self.arguments.push(command.into());
        self
    }

    pub fn build(self) -> Request {
        Request {
            arguments: self.arguments,
        }
    }
}