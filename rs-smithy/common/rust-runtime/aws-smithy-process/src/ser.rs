// basic serialization code
use std::collections::{HashMap};

pub fn serialize_map(map: HashMap<String, String>, delimiter: &str) -> String {
    let mut result = String::new();

    for (key, value) in map {
        result.push_str(&key);
        result.push_str("="); // hard-coded sep
        result.push_str(&value);
        result.push_str(delimiter);
    }

    result.pop();
    result
}

pub fn serialize_vector(vector: Vec<String>) -> String {
    let mut result = String::new();

    for value in vector {
        result.push_str(&value);
        result.push_str(","); // hard-coded delimiter
    }

    result.pop();
    result
}