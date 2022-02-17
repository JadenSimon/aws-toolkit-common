use std::{collections::HashMap, sync::{atomic::AtomicUsize, Arc}, process::Output, pin::Pin, future::Future};

use serde_json::Value;
use toolkits::{error::InvalidValueError, model::FlowSchemaElement};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

type Schema = HashMap<String, FlowSchemaElement>;
type Handler = Box<dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync>;
type UpdateHandler = Box<dyn Fn(&Schema, &HashMap<String, Value>) -> Schema + Send>;

pub struct Flow
{
    id: String,
    schema: Schema,
    state: HashMap<String, Value>,
    handler: Option<Handler>,

    // adding an 'update' handler to support dynamic flows
    // though I think a trait would be better
    update_handler: Option<UpdateHandler>
}

// Maybe the idea of a 'flow' is not a concrete one, and is instead something that needs to be a trait.
impl Flow {
    pub fn new(schema: Schema) -> Self {
        let id = (COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)).to_string();

        Self { handler: None, state: HashMap::new(), schema, id, update_handler: None }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_handler(mut self, handler: Handler) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn set_update_handler(mut self, handler: UpdateHandler) -> Self {
        self.update_handler = Some(handler);
        self
    }

    pub fn update_state(&mut self, key: String, value: Value) -> Result<(), InvalidValueError> {
        if !self.schema.contains_key(&key) {
            return Err(InvalidValueError::builder().message("No key found").build());
        }

        self.state.insert(key, value);
        
        // Update handlers are allowed to mutate the schema as if the schema itself were dynamic
        if let Some(update_handler) = &self.update_handler {
            self.schema = update_handler(&self.schema, &self.state)
        }

        Ok(())
    }

    // Returning a ref is bad since the state can easily go stale
    pub fn get_state(&self) -> &HashMap<String, Value> {
        &self.state
    }

    pub fn get_schema(&self) -> Schema {
        self.schema.clone()
    }

    pub async fn complete(self) -> Option<String> {
        match self.handler {
            Some(handler) => Some(handler(self.state).await),
            None => None,
        }
    }
}


// What's the best way to customize graph construction?
// A trait might make sense, but it's also heavy
// We'll probably need to write a macro to describe a 'state' object 
// given a list of keys and types
//struct FlowNode<T: Resource> {

//}
// might look something like this:
// node::builder()
//  .add_dependency([node1])
//  .add_dependency([node2])
//  ...
//  .build(|state| ...);
// would need to use tuple-types but wouldn't be that bad?
// Node<S, O>
// builder<O>() -> Self<(), O>
// add_dependency<N>(mut self, &other: N) -> Self<(S, N<S>), O>

// This is the 'dynamic' version of `Flow`
// It recomputes the UI graph (or 'schema') after every change in state
// Describing it as a trait is easiest since it allows the most arbitrary logic, though it's not as 'safe'
//
// The 'Flow' idea is pretty similar to something like static websites built off pure HTML
// Things like JS got popular since it allowed reactive websites without the latency of fetching new content
// But the 'fetching' part isn't a concern for us.
trait Flow2 {

}
