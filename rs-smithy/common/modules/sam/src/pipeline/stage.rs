use aws_core::resource::Resource;
use toolkits::model::FlowSchemaElement;
use super::service::{Controller, SamPipeline};

/// Pipeline stages are essentially aggregate resources defined by `pipelineconfig.toml`
/// One could say that generated resources (as defined by the `ManagedStackSource` tag) are 'owned'
/// by the stage, though it's possible to share generated resources between stages.
struct PipelineStage {
    name: String,
}

impl Resource for PipelineStage {
    fn summary(&self) -> toolkits::model::ResourceSummary {
        toolkits::model::ResourceSummary::builder()
            .name(&self.name)
            .iri("aws:sam/pipeline/stage/".to_owned() + &self.name) // IRI should be based off something else, not name
            .resource_type("SAMPipelineStage")
            .build()
    }
}

impl SamPipeline {
    pub fn create_stage_flow(&self) -> aws_flow::model::Flow {
        let name = FlowSchemaElement::builder()
            .resource_type("string")
            .name("Stage name")
            .required(true)
            .description("Name for the stage")
            .relative_order(1)
            .build();

        let region = FlowSchemaElement::builder()
            .resource_type("Region") // TODO: `Region` resource
            .name("Region")
            .required(true) 
            .description("The region where these resources will be created")
            .relative_order(2) // Everything following this param would be a dependency
            .build();

        // the next 5 resources all have default values based off the stage name
        // also, SAM CLI will create them if we don't provide anything

        let pipeline_user = FlowSchemaElement::builder()
            .resource_type("IamUser") // TODO: add resource
            .name("Pipeline User ARN")
            //.required(true) 
            .description("An IAM user used to manage the pipeline")
            .relative_order(3)
            .build();

        let pipeline_execution_role = FlowSchemaElement::builder()
            .resource_type("IamRole") // TODO: add resource
            .name("Pipeline Execution Role ARN")
            //.required(true) 
            .description("An IAM role used to execute the pipeline")
            .relative_order(4)
            .build();

        let cloudformation_execution_role = FlowSchemaElement::builder()
            .resource_type("IamRole") // TODO: add resource
            .name("CloudFormation Execution Role ARN")
            //.required(true) 
            .description("An IAM role used to deploy pipeline resources")
            .relative_order(5)
            .build();

        let artifact_bucket = FlowSchemaElement::builder()
            .resource_type("S3Bucket") // TODO: add resource
            .name("Artifact Bucket")
            //.required(true) 
            .description("Bucket used to store pipeline artifacts")
            .relative_order(6)
            .build();

        // Looks like this one is a list of resources?
        let image_repository = FlowSchemaElement::builder()
            .resource_type("ECRRepository") // TODO: add resource
            .name("ECR Image Repository")
            .description("Repository to store image-based lambdas")
            .relative_order(7)
            .build(); 

        // There would also be a confirmation step, though not sure how to describe that yet

        let mut schema = std::collections::HashMap::new();
        schema.insert("name".to_owned(), name);
        schema.insert("region".to_owned(), region);
        schema.insert("pipeline_user".to_owned(), pipeline_user);
        schema.insert("pipeline_execution_role".to_owned(), pipeline_execution_role);
        schema.insert("cloudformation_execution_role".to_owned(), cloudformation_execution_role);
        schema.insert("artifact_bucket".to_owned(), artifact_bucket);
        schema.insert("image_repository".to_owned(), image_repository);

        async fn create(controller: Controller, state: std::collections::HashMap<String, serde_json::Value>) -> String {
            let mut mapped: std::collections::HashMap<String, String> = state.into_iter().map(|(k, v)| (k, v.as_str().unwrap().to_owned())).collect();
            let input = toolkits::input::RunPipelineBootstrapInput::builder()
                .stage(mapped.get("name").unwrap())
                .region(mapped.get("region").unwrap()) // Region is not required but we'll say it is anyway
                .set_pipeline_user(mapped.remove("pipeline_user"))
                .set_pipeline_execution_role(mapped.remove("pipeline_execution_role"))
                .set_cloudformation_execution_role(mapped.remove("cloudformation_execution_role"))
                .set_bucket(mapped.remove("artifact_bucket"))
                .set_image_repository(mapped.remove("image_repository"))
                .build().unwrap();
            
            run_bootstrap(controller, input).await.expect("Failed to run SAM CLI pipeline bootstrap")
        }

        let controller = self.controller.clone();
        aws_flow::model::Flow::new(schema)
            .set_handler(Box::new(move |state| Box::pin(create(controller.clone(), state))))
    }
}

async fn run_bootstrap(controller: Controller, input: toolkits::input::RunPipelineBootstrapInput) -> Result<String, Box<dyn std::error::Error>> {
    let builder = aws_smithy_process::request::RequestBuilder::new()
        .add_command("pipeline")
        .add_command("bootstrap")
        .add_flag("--no-interactive")
        .add_flag("--no-confirm-changeset"); // we will need to remove this then add a 'confirm' prompt instead

    // TODO: make this nicer??
    let arguments = toolkits::input::RunPipelineBootstrapInput::assemble(builder, input).arguments;
    let request = aws_smithy_process::service::StartProcessRequest {
        command: "sam".to_owned(),
        arguments,
        environment: None,
        working_directory: None,
    };

    let result = controller.execute(request).await?;
    //let result = controller.wait_for2(request).await?;
    //println!("status: {:?}", &result.status.code().map(|x| x.to_string()));
    //println!("stdout: {:?}", std::str::from_utf8(&result.stdout));
    //println!("stderr: {:?}", std::str::from_utf8(&result.stderr));

    Ok(result.upgrade().unwrap().id().into())
    //Ok("1".to_string())
}