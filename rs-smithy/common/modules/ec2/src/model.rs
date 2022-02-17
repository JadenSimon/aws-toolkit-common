use std::{collections::HashMap, sync::{atomic::AtomicUsize, Arc}, os::unix::prelude::OsStringExt};

use async_trait::async_trait;
use aws_core::resource::{List, Create, Resource, Stateful, Registry};
use aws_sdk_ec2::{model::Instance, Client, SdkError};
use serde_json::Value;
use tokio::sync::Mutex;
use toolkits::{error::InvalidValueError, model::{FlowSchemaElement, ResourceSummary}};

/// Top-level model that represents the entire EC2 service associated with a single client configuration.
pub struct Ec2 {
    pub(super) client: Client,
    // I need a thread-safe hash map
    instances: Arc<Mutex<HashMap<String, Arc<Ec2Instance>>>>,
    stale: bool,
}

impl Ec2 {
    pub async fn create() -> Self {
        let config = aws_config::from_env().load().await;

        Self { client: Client::new(&config), instances: Arc::new(Mutex::new(HashMap::new())), stale: true } 
    }

    pub async fn delete_instance(&mut self, instance: Arc<Ec2Instance>) -> Result<(), SdkError<aws_sdk_ec2::error::TerminateInstancesError>> {
        self.client.terminate_instances()
            .instance_ids(instance.id())
            .send()
            .await?;
        Ok(())
    }
}

#[async_trait]
impl List<Ec2Instance> for Ec2 {
    type Error = SdkError<aws_sdk_ec2::error::DescribeInstancesError>;
    
    async fn list(&self) -> Result<Vec<Arc<Ec2Instance>>, Self::Error> {
        match self.stale {
            false => Ok(self.instances.lock().await.values().map(|x| Arc::clone(x)).collect()),
            true => {
                let resp = self.client.describe_instances().send().await?;
                let instances = resp.reservations.unwrap().into_iter().map(|res| {
                    res.instances.unwrap().into_iter().map(|inst| {
                        Arc::new(Ec2Instance::new(inst))
                    }).collect::<Vec<Arc<Ec2Instance>>>()
                }).flatten().collect::<Vec<Arc<Ec2Instance>>>();

                let mut locked_instances = self.instances.lock().await;
                locked_instances.clear();
                for instance in &instances {
                    locked_instances.insert(instance.summary().iri.unwrap().to_owned(), Arc::clone(instance));
                }

                Ok(instances)
            },
        }
    }
}

#[async_trait]
impl Create<Ec2Instance> for Ec2 {
    type Error = SdkError<aws_sdk_ec2::error::RunInstancesError>;
    type Input = HashMap<String, String>;

    // The AWS console has about 7 panes for this flow
    // It's perfectly doable as a DAG, though it might involve supporting a lot of related resources
    // if you want it to actually be good
    async fn create(&mut self, input: Self::Input) -> Result<Arc<Ec2Instance>, Self::Error> {
        let resp = self.client.run_instances()
            .image_id(input.get("image_id").unwrap().split("/").last().unwrap()) // required
            .min_count(1) // required, but we probably won't support it in the UI?
            .max_count(1) // required, see ^
            .instance_type(aws_sdk_ec2::model::InstanceType::A1Medium) // needs to be based off the image...
            .send().await?;

        let instance = Arc::new(Ec2Instance::new(resp.instances.unwrap().pop().unwrap()));
        // Note: `unwrap` on IRI is safe since it _must_ be required
        self.instances.lock().await.insert(instance.summary().iri.unwrap().to_owned(), Arc::clone(&instance));

        Ok(instance)
    }
}

#[async_trait]
impl Registry<Ec2Instance> for Ec2 {
    async fn get_resource(&self, iri: &str) -> Option<Arc<Ec2Instance>> {
        self.instances.lock().await.get(iri).map(Arc::clone)
    }
}

/// This is the server's view of the instance in AWS and should not be conflated with a real instance
pub struct Ec2Instance {
    /// A 'summary' of the instance as described by a list operation.
    /// It's considered a summary since there may be more information available, it's just not provided via
    /// a single API call.
    summary: Instance,
}

impl Ec2Instance {
    pub fn new(summary: Instance) -> Self {
        Self { summary }
    }
    pub fn id(&self) -> String {
        self.summary.instance_id().unwrap().to_owned()
    }
}

impl Resource for Ec2Instance {
    fn summary(&self) -> ResourceSummary {
        ResourceSummary::builder()
            .name(self.summary.instance_id().unwrap().to_owned())
            .detail(self.summary.instance_type().map_or("", |x| x.as_str()))
            .description("An EC2 Instance")
            .iri("aws:ec2/instance".to_owned() + "/" + self.summary.instance_id().unwrap())
            .resource_type("EC2Instance") // maybe do the ARN if we can find it?
            .build()
    }
}

use aws_sdk_ec2::model::InstanceStateName;

// Probably a better way to do this without clones, but I didn't want to have nested matches
fn unwrap_state(state: Option<&aws_sdk_ec2::model::InstanceState>) -> InstanceStateName {
    let default = InstanceStateName::Unknown("unknown".to_owned());
    state.map_or(&default, |x| x.name().unwrap_or(&default)).to_owned()
}

impl Stateful for Ec2Instance {
    fn get_state(&self) -> String {
        match unwrap_state(self.summary.state()) {
            InstanceStateName::Pending => "Pending",
            InstanceStateName::Stopping => "Stopping",
            InstanceStateName::ShuttingDown => "Shutting down",
            InstanceStateName::Stopped => "Stopped",
            InstanceStateName::Terminated => "Terminated",
            InstanceStateName::Running => "Running",
            InstanceStateName::Unknown(state) => return state,
            _ => "Unknown",
        }.to_owned()
    }

    fn is_transient(&self) -> bool {
        match unwrap_state(self.summary.state()) {
            InstanceStateName::Pending => true,
            InstanceStateName::Stopping => true,
            InstanceStateName::ShuttingDown => true,
            InstanceStateName::Stopped => false,
            InstanceStateName::Terminated => false,
            InstanceStateName::Running => false,
            _ => false,
        }
    }
}


// -------------- Flows --------------- //

pub fn create_instance_flow() -> aws_flow::model::Flow {
    let image_step =  toolkits::model::FlowSchemaElement::builder()
        .resource_type("EC2Image")
        .name("EC2 Image")
        .required(true)
        .build();

    let mut schema = HashMap::new();
    schema.insert("image_id".to_string(), image_step);

    async fn create(state: HashMap<String, Value>) -> String {
        let mapped = state.into_iter().map(|(k, v)| (k, v.as_str().unwrap().to_owned())).collect();
        let instance: Arc<Ec2Instance> = Ec2::create().await.create(mapped).await.unwrap();

        instance.summary().iri().unwrap().to_owned()
    }

    aws_flow::model::Flow::new(schema)
        .set_handler(Box::new(|state| Box::pin(create(state))))
}


// ------------------------------------ //
