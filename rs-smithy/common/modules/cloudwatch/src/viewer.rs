use std::ops::Add;

use aws_sdk_cloudwatchlogs::{client, Region, SdkError};
use aws_sdk_cloudwatchlogs::model::{LogGroup, LogStream, OutputLogEvent};
use aws_sdk_cloudwatchlogs::error::GetLogEventsError;
use toolkits::model;


pub struct LogStreamViewer {
    cursor: usize,
    events: Vec<model::OutputLogEvent>, // cached events
    next_token: Option<String>,
    eof: bool,

    // immutable stuff
    client: client::Client,
    log_group_name: String,
    log_stream_name: String,
}

impl LogStreamViewer {
    // TODO: make better id
    pub fn id(&self) -> String {
        self.log_group_name.clone() + "/" + &self.log_stream_name
    }

    pub async fn create(request: toolkits::input::CreateLogStreamViewerInput) -> Self {
        let config = aws_config::from_env().load().await;

        Self { 
            cursor: 0,
            events: Vec::new(),
            log_group_name: request.log_group_name.unwrap(), 
            log_stream_name: request.log_stream_name.unwrap(), 
            client: client::Client::new(&config),

            next_token: None,
            eof: false,
        }
    }
    

    pub async fn scroll_to(&mut self, request: toolkits::input::ScrollToInput) -> toolkits::output::ScrollToOutput {
        let pos = request.position().unwrap() as usize;

        while self.events.len() <= pos && !self.eof {
            let resp = self.client.get_log_events()
                .log_group_name(&self.log_group_name)
                .log_stream_name(&self.log_stream_name)
                .set_limit(Some(100))
                .set_next_token(self.next_token.as_ref().map(|x| x.clone()))
                .start_from_head(true) // if using next_token, you must set this as true ??
                .send().await.unwrap(); // bleh unwrap

            println!("{:?}", resp.events.as_ref().map(|x| x.len()));

            self.events.extend(resp.events.unwrap().iter().map(|event| {
                model::OutputLogEvent::builder()
                    .set_message(event.message.as_ref().map(|x| x.clone()))
                    .set_ingestion_time(event.ingestion_time)
                    .set_timestamp(event.timestamp)
                    .build()
            }));

            println!("{:?}", &resp.next_forward_token);
            println!("{:?}", &resp.next_backward_token);

            self.eof = self.next_token.as_deref() == resp.next_forward_token.as_deref();
            self.next_token = resp.next_forward_token;
        }

        self.cursor = pos;
        let end = std::cmp::min(self.events.len(), pos + 100);
        let events = &self.events[pos..end];

        // not doing any errors for now...
        toolkits::output::ScrollToOutput::builder()
            .cursor(pos.try_into().unwrap())
            .set_events(Some(events.to_vec()))
            .build()
    }
}

/**
 * 
async fn handle(viewers: Arc<Mutex<HashMap::<String, LogStreamViewer>>>, method: String, body: serde_json::Value) -> std::result::Result<impl Reply, Rejection> {
    match method.as_str() {
        "create" => {
            println!("{:?}", &body);

            let request: CreateLogStreamViewerInput = serde_json::from_value(body).unwrap();
            let viewer = LogStreamViewer::create(request).await;
            let id = viewer.id();
            viewers.lock().await.insert(id.clone(), viewer);
            let response = CreateLogStreamViewerOutput::builder()
                .log_stream_viewer_id(id)
                .build();

            Ok(warp::reply::json(&response))
        },  
        "get" => {
            let request: CreateLogStreamViewerInput = serde_json::from_value(body).unwrap();
            let viewer = LogStreamViewer::create(request).await;
            let response = CreateLogStreamViewerOutput::builder()
                .log_stream_viewer_id(viewer.id())
                .build();

            Ok(warp::reply::json(&response))
        },
        "scroll" => {
            let request: ScrollToInput = serde_json::from_value(body).unwrap();

            if let Some(viewer) = viewers.lock().await.get_mut(request.log_stream_viewer_id.as_ref().unwrap()) {
                let response = viewer.scroll_to(request).await;

                return Ok(warp::reply::json(&response));
            }

            Err(warp::reject())
        },
        _ => panic!("Invalid method!")
    }
}
 */