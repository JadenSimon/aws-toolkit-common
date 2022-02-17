use std::collections::HashMap;
use aws_smithy_json_rpc::request::Request;


#[derive(Default, Debug)]
struct ParseError;



impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", "foo")
    }
}

impl std::error::Error for ParseError {}

// Adds the LSP header to a JSON RPC request: https://microsoft.github.io/language-server-protocol/specification
pub fn to_lsp_message(request: Request) -> Result<String, Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string(&request)?;
    Ok(format!("Content-Length: {}\r\n\r\n{}", serialized.len(), &serialized))
}

pub fn from_lsp_message(message: &str) -> Result<Request, Box<dyn std::error::Error>> {
    let mut header = HashMap::<&str, &str>::new();
    let mut lines = message.split("\r\n");

    // Header key/values should always be split on ': '
    while let Some(line) = lines.next() {
        // Consecutive new lines
        if line.len() == 0 { 
            break; 
        }

        let section = line.split(": ");

        if let [key, value] = section.take(2).collect::<Vec<&str>>()[..2] {
            header.insert(key, value);
        } else {
            return Err(Box::new(ParseError::default()));
        }
    }

    let content = &lines.collect::<Vec<&str>>().join("\r\n");
    // you would normally check if content's length is the same as `Content-Length` now
    Ok(serde_json::de::from_str(content)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let input = "Content-Length: 60\r\n\r\n{\"method\": \"bar\", \"id\": \"0\", \"jsonrpc\": \"2.0\", \"params\": {}}";
        let parsed = from_lsp_message(input).expect("Failed to parse string");

        assert_eq!(parsed.method, "bar");
    }

}