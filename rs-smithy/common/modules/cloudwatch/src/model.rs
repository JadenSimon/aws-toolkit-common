use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use aws_sdk_cloudwatchlogs::{client, Region, SdkError};
use aws_sdk_cloudwatchlogs::model::{LogGroup, LogStream, OutputLogEvent};
use aws_sdk_cloudwatchlogs::error::GetLogEventsError;

use futures::future::{FutureExt, Shared};


// Current thinking is that this ought to take a 'layered' approach where the backend is essentially an 'adapter'
// for each toolkit. Toolkits need to act as thin clients, knowing very little about the underlying model or service.
// That is, the backend must be opinionated (to some extent) about presentation content as well as what operations
// can be performed on any given resource. But we do not expose the entire model. 
//
// Take, for example, `CreateLogGroup`. What are all the valid options?
// * log group name (required)
// * KMS key (optional)
// * Tags (optional)
//
// So when a user tries to create a new group in the UI, we present them with these configuration values
// However, currently the implementations would know _exactly_ which service (and specifically, operation)
// these values are being used for. This creates impedance towards code reuse; our UI elements become tighly
// bound to service operations without lots of abstraction on the frontend. The current modeling always offers
// no guidance on how to present these 3 configuration values - the interface is ephemeral. 
//
// Let us instead reverse the direction of this flow. Instead of the frontend prompting users first for values,
// the frontend should immediately request the backend to create a log group. That is, the frontend only sees a 
// 'create' operation that accepts no parameters (or perhaps some basic global ones). The backend responds 
// with a stateful 'session' object that describes a flow to create a log group. It contains an ID, a state ID,
// a schema for the current state, and a currently assigned state which may contain defaults.
//
// Clients use the schema to render UI (however they see fit), then communicate back to the server anytime state
// changes. The server either responds successfully with a new state ID (and possibly schema), or rejects the request
// with an error. The error message is _not_ apart of the state, so it is up to clients to render these messages.
//
// Now, if we assume that there is no external source of mutations, then clients can stay in-sync by always passing
// along a 'state ID' with every request. The server can compare its current state with the token, letting the client
// know if it is out-of-sync. This is extremely similar to how caching works in modern web-browsers via ETags. 
// 
// Of course, creating a log group causes a change in a different resource: the CWL service itself. This is rendered
// apart of the explorer tree in most toolkits. How do we go about updating the tree? Currently, each toolkit just refreshes
// the tree after the flow is complete since they 'know' that a new log group means the tree has changed. But that
// knowledge can be shared! 
//
// Being able to 'refresh' other resources requires some mechanism of communicating that things have changed. In the previous
// example, we were passing along tokens apart of requests. State is 'self-contained' since the flow itself is mutating
// state, so notifying changes is easy: just have it apart of the response. For 'other' resource, the either needs an event-based
// solution or a 'global' mechanism to alert refreshes via request/response. 
//
// The vast number of possible connections between resources means that a notification system is more viable. This would lift
// the burden of clients from having to figure out when it should refresh; the server just sends an event saying that some resource
// has changed, so it should request for a fresh description and refresh the UI.
//
// This 'thin client' approach is practically 'all or nothing' in the sense that we separate IDEs completely from service calls.
// The backend _must_ provide everything the client needs, otherwise encapsulation is shattered. The goal here is that services
// are only modeled for UI purposes in a single place: the backend. When the client wants to create a new log group, it doesn't
// know that the prompt for a name maps 1:1 with `logGroupName`; it just sees that the server wants a string. We, of course, want
// to include additional information about how that prompt might be presented, such as a description, messsage, etc. But we will
// not expose how this specific piece of information relates to the bigger picture. That is only relevant to the backend. 

pub async fn list_groups() -> Vec<LogGroup> {
    let shared_config = aws_config::from_env().region(Region::new("us-west-2")).load().await;
    let client = client::Client::new(&shared_config);
    let mut response = client.describe_log_groups().send().await.unwrap();

    response.log_groups.take().expect("No log groups found")
}


struct StoredLogGroup {
    pub(super)inner: LogGroup,
}


type LogStreamResult = Result<Vec<OutputLogEvent>, Arc<SdkError<GetLogEventsError>>>;
type SharedFut = Shared<Pin<Box<dyn Future<Output = LogStreamResult> + Send>>>;

struct StoredLogStream {
    inner: LogStream,

    /// Pagination token
    token: Option<String>,

    /// Shared future
    future: Option<SharedFut>,

    /// Event indexed off token
    events: HashMap<String, Vec<OutputLogEvent>>,
}

// Not planning on making this thread-safe anytime soon
/// Singular-source for CWL groups/streams
pub struct CloudWatchLogsRegistry {
    /// Registries are per-region; changing credentials would make this registry stale
    client: client::Client,

    /// Indexed off the name
    groups: HashMap<String, StoredLogGroup>,

    /// Streams are indexed as {group_name}/{stream_name}
    streams: HashMap<String, StoredLogStream>,
}   

impl CloudWatchLogsRegistry {
    pub fn new(config: &aws_config::Config) -> Self {
        Self { groups: HashMap::new(), streams: HashMap::new(), client: client::Client::new(config) }
    }

    pub async fn list_all_groups(&mut self) -> Vec<&LogGroup> {
        if self.groups.len() > 0 {
            return self.groups.values().map(|g| &g.inner).collect();
        }

        let mut groups = Vec::new();

        let mut token: Option<String> = None;
        loop {
            let mut response = self.client.describe_log_groups().set_next_token(token.clone()).send().await.unwrap();
            groups.extend(response.log_groups.take().unwrap());
            token = response.next_token().take().map_or(None, |x| Some(x.to_owned()));

            if token.is_none() { break; }
        }

        for group in groups {
            if let Some(name) = group.log_group_name() {
                self.groups.insert(name.to_owned(), StoredLogGroup { inner: group });
            }
        }

        self.groups.values().map(|g| &g.inner).collect()
    }

    // no caching
    pub async fn list_all_streams(&mut self, group_name: &str) -> Vec<&LogStream> {
        let mut streams = Vec::new();

        let mut token: Option<String> = None;
        loop {
            let mut response = self.client.describe_log_streams()
                .descending(true) // could change this to do 'tailing'
                .order_by(aws_sdk_cloudwatchlogs::model::OrderBy::LastEventTime)
                .log_group_name(group_name)
                .set_next_token(token.clone())
                .send().await.expect("Failed to get log streams");

            streams.extend(response.log_streams.take().unwrap());
            token = response.next_token().take().map_or(None, |x| Some(x.to_owned()));

            if token.is_none() { break; }
        }

        for stream in streams {
            if let Some(name) = stream.log_stream_name() {
                if let Some(old_stream) = self.streams.remove(name) {
                    self.streams.insert(String::from_str(group_name).unwrap() + "/" + name, StoredLogStream { 
                        inner: stream, 
                        token: old_stream.token,
                        future: old_stream.future,
                        events: old_stream.events,
                    });
                } else {
                    self.streams.insert(String::from_str(group_name).unwrap() + "/" + name, StoredLogStream { 
                        inner: stream, 
                        token: None,
                        future: None,
                        events: HashMap::new(),
                    });
                }
            }
        }

        self.streams.values().map(|g| &g.inner).collect()
    }


    /// Returns true if a stream is ready to be read from, false otherwise
    pub async fn poll_stream(&mut self, group_name: &str, stream_name: &str) -> Option<LogStreamResult> {
        let key = String::from_str(group_name).unwrap() + "/" + stream_name;

        match self.streams.get_mut(&key) {
            Some(stream) => {
                match &stream.future {
                    Some(future) => {
                        let fut = future.clone();
                        Some(fut.await)
                    },
                    None => {
                        // create a new future
                        let fut = self.client.get_log_events()
                            .log_group_name(group_name)
                            .log_stream_name(stream_name)
                            .send();
                        let shared: SharedFut = fut.map(|r| r.map(|o| o.events.unwrap()).map_err(Arc::new)).boxed().shared();
                        let copy = shared.clone();
                        stream.future = Some(shared);
                        Some(copy.await)
                    },
                }
            },
            None => None,
        }
    }
}

// PoC integration:
// 1. filtering
// 2. caching
// 3. event stream
// 4. Insights?
// 
// first 3 things would cover the current explorer impl. in VS Code
// but we need to do _more_ than that. Insights seems reasonable
