use std::error::Error;

// Normally I'd just do a static router, but I'll make it dynamic for fun
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct RouterError {
    message: Option<String>,
}

impl Error for RouterError {}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RouterError")?;
        if let Some(msg) = &self.message {
            write!(f, ": {}", msg)?;
        }
        Ok(())
    }
}

// TODO: what should this look like?
pub struct Router {

}