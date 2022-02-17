use std::fmt;

#[derive(Debug, Clone)]
pub struct StubError;

impl fmt::Display for StubError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "stub")
    }
}