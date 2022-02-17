use crate::request::Request;
use tower::Service;
use std::error::Error;
use serde_json::Value;
use std::future::Future;

// This service is dependent on some sort of a 'network' (i.e. transport protocol) service
pub struct JsonRpcService<S> {
    service: S,
    id_counter: usize,
}

// Just stuff Response here for now
pub struct Response {
    id: String, // Technically can be a string but eh
    result: Value,
}

// Three types of errors here:
// 1. Transport layer
// 2. Serialization/deserialization
// 3. Error from the server itself
// Just make these dynamic...
enum ServiceError {
    SerdeJson(serde_json::Error),
    Transport(std::io::Error),
    Server(ResponseError),
}

#[derive(Debug)]
struct ResponseError {
    code: i32,
    message: String, // Is this required?
    data: Option<Value>, // Need to check for null?
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Error for ResponseError {}

// This service is flawed in the sense that it assumes the inner service will know how
// to piece the data structures back together. Basically, this would only work if we
// assume a one way communication channel
impl<S> Service<Request> for JsonRpcService<S>
where 
    S: Service<String, Response = String, Error = Box<dyn Error>>,
    S::Future: Send + 'static,
{
    type Response = self::Response;
    type Error = Box<dyn Error>;
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let serialized = serde_json::to_string(&req).unwrap();
        let resp = self.service.call(serialized);

        let p = async move {
            let result = resp.await?;
            let parsed = serde_json::from_str::<Value>(&result)?;

            match parsed {
                Value::Object(mut map) => {
                    let result = map.remove("result").unwrap_or(serde_json::Value::Null);
                    let id = map.remove("id").unwrap_or(serde_json::Value::Null);
                    // TODO: if id does not exist then it is invalid
                    Ok(Response {
                        id: id.as_str().unwrap().to_owned(),
                        result,
                    })
                },
                _ => return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "").into()),
            }
        };

        Box::pin(p)
    }
}