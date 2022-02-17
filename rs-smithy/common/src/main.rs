use aws_core::resource::{Resource, List, Registry, Stateful};
use aws_ec2::model::{Ec2, Ec2Instance};
use aws_ec2::images::Ec2Image;
use aws_flow::model::Flow;
use aws_core::tools::ToolController;
use aws_sam::pipeline::service::SamPipeline;
// use aws_cloudwatch::viewer::LogStreamViewer;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex, fs::read};
use toolkits::{input::{StartFlowInput, GetFlowStateInput, UpdateFlowStateInput, GetResourcesInput, GetFeaturesInput}, output::{StartFlowOutput, GetFlowStateOutput, UpdateFlowStateOutput, GetResourcesOutput, GetFeaturesOutput}, model::{FlowSchemaElement, ResourceSummary, Feature}};
use tower::Service;
use serde::{Serialize, Deserialize};
use warp::ws::{Ws, WebSocket, Message};
use std::{io::{self, Write}, collections::HashMap, sync::{Arc}, path::PathBuf};
use futures::{FutureExt, StreamExt, SinkExt};

//use aws_smithy_process::request;
use aws_smithy_json_rpc::request;

use warp::{Filter, Rejection, Reply};

mod watcher;
mod registry;
mod lsp;
//path = "../../../../../aws-smithy-process" 

#[derive(Serialize, Deserialize)]
struct Empty;

type Flows = Arc<Mutex<HashMap::<String, Flow>>>;

#[derive(Clone)]
struct Globals {
    ec2: Arc<Mutex<Ec2>>,
    flows: Flows,
    sam_pipeline: Arc<Mutex<SamPipeline>>,
    tool_controller: Arc<ToolController>,
}

impl Globals {
    pub async fn init() -> Self {
        let tool_controller = Arc::new(ToolController::new());

        Self {
            ec2: Arc::new(Mutex::new(Ec2::create().await)),
            flows: Arc::new(Mutex::new(HashMap::<String, Flow>::new())),
            sam_pipeline: Arc::new(Mutex::new(SamPipeline::init(Arc::clone(&tool_controller)).await.unwrap())),
            tool_controller,
        }
    }

    pub fn ec2(&self) -> Arc<Mutex<Ec2>> {
        Arc::clone(&self.ec2)
    }

    pub fn flows(&self) -> Flows {
        Arc::clone(&self.flows)
    }

    pub fn tool_controller(&self) -> Arc<ToolController> {
        Arc::clone(&self.tool_controller)
    }
}

async fn handle(globals: Arc<Globals>, method: String, body: serde_json::Value) -> std::result::Result<impl Reply, Rejection> {
    let flows = globals.flows();

    let example_flow = PathBuf::from("/Users/sijaden/telemetry/rs-smithy/sam/src/flows/create-log-group.json");
    let contents = read(example_flow).await.unwrap();
    let utf8 = std::str::from_utf8(&contents).map_or(Err("Unable to read flow"), Ok).unwrap();
    let schema = serde_json::de::from_str::<HashMap::<String, FlowSchemaElement>>(utf8).unwrap();

    match method.as_str() {
        "create" => {
            let request: StartFlowInput = serde_json::from_value(body).unwrap();
            let flow = Flow::new(schema.clone());
            let id = flow.id().to_string();

            flows.lock().await.insert(id.clone(), flow);
            let response = StartFlowOutput::builder()
                .id(id)
                .set_schema(Some(schema.into()))
                .build();

            Ok(warp::reply::json(&response))
        },  
        "get" => {
            let request: GetFlowStateInput = serde_json::from_value(body).unwrap();
            if let Some(flow) = flows.lock().await.get(request.id().unwrap()) {
                let response = GetFlowStateOutput::builder().build();

                return Ok(warp::reply::json(&response));
            }

            Err(warp::reject())

        },
        "update" => {
            let request: UpdateFlowStateInput = serde_json::from_value(body).unwrap();

            if let Some(flow) = flows.lock().await.get_mut(request.id().unwrap()) {
                let status = flow.update_state(request.key.unwrap(), serde_json::from_str(&request.value.unwrap()).unwrap());
                let response = UpdateFlowStateOutput::builder().build();

                return match status {
                    Ok(resp) => Ok(warp::reply::json(&resp)),
                    Err(resp) => Err(warp::reject()),
                }
            }

            Err(warp::reject())
        },
        "completeFlow" => {
            let request: toolkits::input::CompleteFlowInput = serde_json::from_value(body).unwrap();

            // wouldn't support retries
            if let Some(flow) = flows.lock().await.remove(request.id().unwrap()) {
                let result = flow.complete().await.unwrap_or_default();
                let resp = toolkits::output::CompleteFlowOutput::builder()
                    .data(result)
                    .build();

                return Ok(warp::reply::json(&resp));
            }

            Err(warp::reject())
        },
        "getFlowSchema" => {
            let request: toolkits::input::GetFlowSchemaInput = serde_json::from_value(body).unwrap();

            // wouldn't support retries
            if let Some(flow) = flows.lock().await.get(request.id().unwrap()) {
                let resp = toolkits::output::GetFlowSchemaOutput::builder()
                    .set_schema(Some(flow.get_schema()))
                    .build();

                return Ok(warp::reply::json(&resp));
            }

            Err(warp::reject())
        },
        "getResources" => {
            let request: GetResourcesInput = serde_json::from_value(body).unwrap();
            let resources = match request.scope.as_ref().map(|x| x.as_str()) {
                Some("aws:ec2") => {
                    let list: Vec<Arc<Ec2Instance>> = globals.ec2().lock().await.list().await.unwrap();
                    list.iter().map(|x| x.summary()).collect()
                },
                None => vec![
                    ResourceSummary::builder()
                        .name("EC2")
                        .resource_type("EC2") // EC2 is a type of service; should this information be relayed to the client?
                        .iri("aws:ec2")
                        .build(),
                    ResourceSummary::builder()
                        .name("SAM-CLI")
                        .resource_type("SAMCLI")
                        .iri("aws:sam-cli")
                        .build()
                ],
                _ => Vec::new(),
            };
            let response = GetResourcesOutput::builder()
                .set_resources(Some(resources))
                .build();

            Ok(warp::reply::json(&response))
        },
        "getFeatures" => {
            let request: GetFeaturesInput = serde_json::from_value(body).unwrap();
            let features = match request.resource_type.unwrap().as_str() {
                "EC2" => vec![
                    // It'd be better if feature types aligned more along Rust traits
                    // Of course that would mean the types would have to be generic
                    // There's still the question of whether create/list features should be on the 'parent'
                    // or on the individual resource. Having it all on a resource means we'd need a separate
                    // way to describe 'ownership', at least for tree views.
                    //
                    // But does the client really need to know exactly which service a resource came from?
                    // Sure, it helps for creating certain UIs. But beyond that there's little reason to
                    // expose that information.
                    //
                    // If we want things to feel more connected, we can start by freeing the toolkits from
                    // worrying about individual clients. Once those logical divisions are gone, more effort
                    // can go towards thinking about interactions between resources instead of data flow.

                    Feature::builder()
                        .feature_type("List")
                        .name("List EC2 instances")
                        .id("list-ec2-instances")
                        .build(),
                    Feature::builder()
                        .feature_type("Create") // This is the only user-facing feature out of the bunch
                        .name("Create EC2 instance")
                        .id("create-ec2-instance")
                        .build(),
                    Feature::builder()
                        .feature_type("List") // 'List'/'Create'/etc. would fall under a abstract 'Operation' type?
                        .name("List EC2 images")
                        .id("list-ec2-images")
                        .build()
                ],
                "EC2Instance" => vec![
                    // This feature isn't really meant for direct user interaction (like 'list')
                    // But it still falls under the abstraction
                    Feature::builder()
                        .feature_type("Facet")
                        .name("Stateful Resource") // maybe name should be optional?
                        .id("stateful-resource") // While there is only 1 `stateful-resource` feature, it may exist in multiple types
                        .build(),
                    Feature::builder()
                        .feature_type("Operation")
                        .name("Delete")
                        .id("delete-resource")
                        .build()
                ],
                "SAMCLI" => vec![
                    Feature::builder()
                        .feature_type("Create")
                        .name("Create SAM Pipeline")
                        .id("create-sam-pipeline")
                        .description("Not implemented")
                        .build(),
                    Feature::builder()
                        .feature_type("Create")
                        .name("Create SAM Pipeline Stage")
                        .id("create-sam-pipeline-stage")
                        .build()
                ],
                "SpawnedTool" => vec![ // SpawnedTool should probably be marked as 'Streamable' or something?
                    Feature::builder()
                        .feature_type("Operation") // `Operation` is kind of an arbitrary type
                        .name("Forward Streams") 
                        .id("forward-streams") // not impl. for the RPC side, only WS
                        .build(),
                ],
                _ => Vec::new(),
            };
            let response = GetFeaturesOutput::builder()
                .set_features(Some(features))
                .build();

            Ok(warp::reply::json(&response))
        },
        "runFeature" => {
            let request: toolkits::input::RunFeatureInput = serde_json::from_value(body).unwrap();
            let target = request.target().unwrap();

            // we will need a global registry IRI -> concrete resource
            // it doesn't necessarily have to store them, but we need a way to acquire them

            match request.feature().unwrap() {
                "list-ec2-images" => {
                    println!("Running list images on {:?}", &target);
                    // assumption: target refers to EC2
                    let list: Vec<Arc<Ec2Image>> = globals.ec2().lock().await.list().await.unwrap();

                    let images: Vec<ResourceSummary> = list.iter().map(|x| x.summary()).collect();
                    println!("Found {:?} images", &images.len());

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::List(images))
                        .build();

                    
                    Ok(warp::reply::json(&response))
                },
                "create-ec2-instance" => {
                    let flow = aws_ec2::model::create_instance_flow();
                    let id = flow.id().to_string();
                    let schema = flow.get_schema();

                    flows.lock().await.insert(id.clone(), flow);

                    let flow_resp = toolkits::model::StartFlowOutput::builder()
                        .id(id)
                        .set_schema(Some(schema))
                        .build();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::Create(flow_resp))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "stateful-resource" => {
                    if let Some(instance) = globals.ec2().lock().await.get_resource(target).await {
                        let facet = toolkits::model::StatefulResourceFacet::builder()
                            .state(instance.get_state())
                            .transient(instance.is_transient())
                            .build();
                        
                        let response = toolkits::output::RunFeatureOutput::builder()
                            .data(toolkits::model::RunFeatureData::Facet(facet))
                            .build();
                        
                        return Ok(warp::reply::json(&response));
                    }

                    // no resource found!
                    Err(warp::reject())
                },
                "create-sam-pipeline" => {
                    let flow = globals.sam_pipeline.lock().await.create_flow();
                    let id = flow.id().to_string();
                    let schema = flow.get_schema();

                    flows.lock().await.insert(id.clone(), flow);

                    let flow_resp = toolkits::model::StartFlowOutput::builder()
                        .id(id)
                        .set_schema(Some(schema))
                        .build();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::Create(flow_resp))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "create-sam-pipeline-stage" => {
                    let flow = globals.sam_pipeline.lock().await.create_stage_flow();
                    let id = flow.id().to_string();
                    let schema = flow.get_schema();

                    flows.lock().await.insert(id.clone(), flow);

                    let flow_resp = toolkits::model::StartFlowOutput::builder()
                        .id(id)
                        .set_schema(Some(schema))
                        .build();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::Create(flow_resp))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "list-sam-pipeline-templates" => {
                    let list = globals.sam_pipeline.lock().await.list().await.unwrap();
                    let templates: Vec<ResourceSummary> = list.iter().map(|x| x.summary()).collect();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::List(templates))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "list-s3-buckets" => {
                    let list = aws_sam::pipeline::resources::S3::create().await.list().await.unwrap();
                    let resources: Vec<ResourceSummary> = list.iter().map(|x| x.summary()).collect();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::List(resources))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "list-iam-roles" => {
                    let list: Vec<Arc<aws_sam::pipeline::resources::Role>> = aws_sam::pipeline::resources::IAM::create().await.list().await.unwrap();
                    let resources: Vec<ResourceSummary> = list.iter().map(|x| x.summary()).collect();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::List(resources))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "list-iam-users" => {
                    let list: Vec<Arc<aws_sam::pipeline::resources::User>> = aws_sam::pipeline::resources::IAM::create().await.list().await.unwrap();
                    let resources: Vec<ResourceSummary> = list.iter().map(|x| x.summary()).collect();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::List(resources))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "list-ecr-repositories" => {
                    let list = aws_sam::pipeline::resources::ECR::create().await.list().await.unwrap();
                    let resources: Vec<ResourceSummary> = list.iter().map(|x| x.summary()).collect();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::List(resources))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "list-spawned-tools" => {
                    let list = globals.tool_controller().list().await.unwrap();
                    let resources: Vec<ResourceSummary> = list.iter().map(|x| x.summary()).collect();

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::List(resources))
                        .build();
                    
                    Ok(warp::reply::json(&response))
                },
                "delete-resource" => {
                    // hard-coded to only do ec2
                    let ec2 = globals.ec2();
                    let mut client = ec2.lock().await;
                    if let Some(instance) = client.get_resource(target).await {
                        client.delete_instance(instance).await.unwrap()
                    }

                    let response = toolkits::output::RunFeatureOutput::builder()
                        .data(toolkits::model::RunFeatureData::List(vec![]))
                        .build();

                    Ok(warp::reply::json(&response))

                },
                _ => Ok(warp::reply::json(&Empty {}))
            }
        },
        _ => panic!("Invalid method!"),
    }
}

// needed a way to manage sockets
// will also need a way to manage processes
// TODO: rename `aws-smithy-xxx` crates to something better
// also rename `process` to `tools` ?
struct Sockets {}

async fn handle_ws(globals: Arc<Globals>, method: String, socket: WebSocket) {
    println!("Received: {:?}", method);

    // TODO: need a good way to manage these
    if let Some((mut stdout, mut stdin, mut stderr)) = globals.tool_controller().take_buffered_streams(&method).await {
        let (mut tx, mut rx) = socket.split();

        let t1 = tokio::task::spawn(async move {
            let mut buf = [0; 10000];
            while let Ok(len) = stdout.read(&mut buf[..]).await {
                if len > 0 {
                    println!("process stdout: {:?}", std::str::from_utf8(&buf[..len]).unwrap());
                    // TODO: err handling
                    tx.send(Message::binary(&buf[..len])).await.expect("Failed to send to websocket");
                } else {
                    println!("process stdout EOF");
                    break;
                }
            }
        });

        let t2 = tokio::task::spawn(async move {
            let mut buf = [0; 10000];
            while let Ok(len) = stderr.read(&mut buf[..]).await {
                if len > 0 {
                    println!("process stderr: {:?}", std::str::from_utf8(&buf[..len]));
                } else {
                    println!("process stderr EOF");
                    break;
                }
            }
        });

        let t3 = tokio::task::spawn(async move {
            while let Some(result) = rx.next().await {
                // TODO: err handling, clean-up etc.
                match result {
                    Ok(message) => {
                        if message.is_close() {
                            println!("Got close message");
                            drop(stdin);
                            break;
                        }
                        stdin.write(message.as_bytes()).await.expect("Failed to write to stdin")
                    },
                    Err(error) => return Err(error),
                };
            };

            Ok(())
        });

        // I think tasks can be cancelled?
        if let Err(error) = tokio::try_join!(t1, t2, t3) {
            println!("Uh oh! A task failed: {:?}", error);
        } else {
            println!("Finished forwarding for {:?}", method);
        }
    }
}

#[tokio::main]
async fn main() {
    let globals = Arc::new(Globals::init().await);
    let ws_globals = Arc::clone(&globals);

    let rpc = warp::post()
        .and(warp::path("rpc"))
        .and(warp::path::param::<String>())
        .and(warp::body::json())
        .and_then(move |method, body| handle(globals.clone(), method, body));
    
    // just gonna use only this path for prototyping process stdio streams
    let ws = warp::path("ws")
        .and(warp::path::param::<String>())
        .and(warp::ws())
        .map(move |method: String, ws: Ws| {
            let handle = ws_globals.clone();
            // method is currently assumed to be a `SpawnedTool` id
            ws.on_upgrade(move |websocket| {
                handle_ws(handle, method, websocket)
            })
        });

    warp::serve(rpc.or(ws))
        .run(([127, 0, 0, 1], 7812))
        .await;
}

