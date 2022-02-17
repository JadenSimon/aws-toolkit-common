// everything is a resource!

use async_trait::async_trait;
use aws_smithy_process::service::StartProcessRequest;
use toolkits::model::ResourceSummary;
use tokio::process::{Child, ChildStdout, ChildStderr, ChildStdin};
use tokio::io::{Result, BufReader, BufWriter};
use tokio::sync::Mutex;
use std::borrow::{BorrowMut};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::process::Output;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::resource::{Resource, List};

// assumption: none of the parts will contain colons
#[derive(Clone)]
pub struct SpawnedToolId {
    tool_id: String,
    version: String,
    unid: String, // PIDs are not used since they are not strictly unique
}

impl SpawnedToolId {
    pub fn new(tool_id: impl Into<String>, version: impl Into<String>, unid: impl Into<String>) -> Self {
        Self { tool_id: tool_id.into(), version: version.into(), unid: unid.into() }
    }
}

/// Display is for user facing things and should not be used as a reference point for IRIs
impl Display for SpawnedToolId {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let s: String = self.into();
        write!(f, "Tool info: {}", s) // TODO: finish this
    }
}

impl From<&SpawnedToolId> for String {
    fn from(id: &SpawnedToolId) -> String {
        format!("{}:{}:{}", &id.tool_id, &id.version, &id.unid)
    }
}

impl TryFrom<&str> for SpawnedToolId {
    type Error = String;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let mut parts = s.split(":");
        let tool_id = parts.next().map_or(Err("No tool ID found"), Ok)?;
        let version = parts.next().map_or(Err("No version found"), Ok)?;
        let unid = parts.next().map_or(Err("No unid found"), Ok)?;

        match parts.next() {
            Some(rem) => Err(format!("Input contained too many delimiters, remainder {}", rem)),
            None => Ok(SpawnedToolId::new(tool_id, version, unid)),
        }
    }
}

pub struct SpawnedTool {
    id: SpawnedToolId,
    child: Child,
}

impl SpawnedTool {
    pub fn new(id: SpawnedToolId, child: Child) -> Self {
        Self { id, child }
    }

    pub fn id(&self) -> &SpawnedToolId {
        &self.id
    }

    fn inner(self) -> Child {
        self.child
    }
}

impl Resource for SpawnedTool {
    fn summary(&self) -> ResourceSummary {
        ResourceSummary::builder()
            .name("pid: ".to_string() + &Child::id(&self).map_or("dead".to_owned(), |x| x.to_string()))
            .resource_type("SpawnedTool")
            .iri(&self.id)
            .build()
    }
}

impl Deref for SpawnedTool {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}


// Manages all spawned processes. 
// Means we'll have to pass this around...
pub struct ToolController {
    counter: AtomicUsize,
    tools: Arc<Mutex<HashMap<String, Arc<SpawnedTool>>>>,
}

type BufferedStdio = (BufReader<ChildStdout>, BufWriter<ChildStdin>, BufReader<ChildStderr>);

impl ToolController {
    pub fn new() -> Self {
        Self { tools: Arc::new(Mutex::new(HashMap::new())), counter: AtomicUsize::new(0) }
    }

    pub async fn get_resource(&self, id: &str) -> Option<Weak<SpawnedTool>> {
        self.tools.lock().await.get(id).map(Arc::downgrade)
    }

    // XXX: breaks encapsulation (and buggy lol)
    pub async fn take_resource(&self, id: &str) -> Option<SpawnedTool> {
        if let Some(Ok(tool)) = self.tools.lock().await.remove(id).map(Arc::try_unwrap) {
            return Some(tool);
        }

        None
    }

    pub async fn take_buffered_streams(&self, id: &str) -> Option<BufferedStdio> {
        match self.take_resource(id).await.map(SpawnedTool::inner) {
            Some(mut child) => Some((
                BufReader::new(child.stdout.take().unwrap()),
                BufWriter::new(child.stdin.take().unwrap()),
                BufReader::new(child.stderr.take().unwrap()),
            )),
            _ => None,
        }
    }

    pub async fn execute(&self, request: StartProcessRequest) -> Result<Weak<SpawnedTool>> {
        let (_id, tool) = self.spawn(request).await?;

        Ok(Arc::downgrade(&tool))
    }

    pub async fn wait_for(&self, request: StartProcessRequest) -> Result<Output> { // Just return output?
        let (id, _tool) = self.spawn(request).await?;

        if let Ok(child) = Arc::try_unwrap(self.tools.lock().await.borrow_mut().remove(&id).unwrap()) {
            let output = child.inner().wait_with_output().await?;
            return Ok(output);
        }

        // make custom err or something, maybe just panic ?
        Err(tokio::io::Error::new(tokio::io::ErrorKind::AddrInUse, "error"))
    }

    async fn spawn(&self, request: StartProcessRequest) -> Result<(String, Arc<SpawnedTool>)> {
        let child = request.execute()?;
        // need to add metadata in `StartProcessRequest?`
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let unid = self.counter.fetch_add(1, Ordering::Relaxed).to_string() + &time.as_millis().to_string(); 
        let id = SpawnedToolId::new("sam", "unknown", unid);
        let tool = Arc::new(SpawnedTool::new(id.clone(), child));
        self.tools.lock().await.insert((&id).into(), Arc::clone(&tool));

        Ok(((&id).into(), tool))
    }
}

#[async_trait]
impl List<SpawnedTool> for ToolController {
    type Error = Infallible;

    async fn list(&self) -> std::result::Result<Vec<Arc<SpawnedTool>>, Self::Error> {
        Ok(self.tools.lock().await.values().map(|x| Arc::clone(&x)).collect())
    }
}