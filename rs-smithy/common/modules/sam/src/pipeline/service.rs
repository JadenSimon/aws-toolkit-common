use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use async_trait::async_trait;
use aws_core::resource::{Resource, List};
use aws_core::tools::ToolController;

use super::flow::{FlowContext, list_templates, clone_templates, parse_context};
use super::mapper::generate_schema;
use super::model::{QuestionsManifest, PipelineTemplate};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type Controller = Arc<ToolController>;

// TODO: re-export
// TODO: make thread-safe, probably generically
pub struct SamPipeline {
    // The below two are just for caching
    templates: Vec<Arc<PipelineTemplate>>,
    manifests: Arc<HashMap<String, QuestionsManifest>>, // maps template id -> questions

    pub(super) controller: Controller,
}

impl SamPipeline {
    pub async fn init(controller: Controller) -> Result<Self> {
        let template_root_dir = clone_templates().await?;
        let raw_templates = list_templates(&template_root_dir).await?;
        let mut manifests = HashMap::new();
        let mut templates = Vec::new();

        for t in raw_templates {
            let template_path = template_root_dir.join(&t.location);
            let ctx = parse_context(&template_path).await?;

            let template: PipelineTemplate = t.into();
            manifests.insert(template.summary().iri.unwrap().to_owned(), ctx.questions_manifest);
            // TODO: fix `into` to make a good name
            templates.push(Arc::new(template));
        }

        Ok(SamPipeline { manifests: Arc::new(manifests), templates, controller })
    }

    pub fn create_flow(&self) -> aws_flow::model::Flow {
        let template_step =  toolkits::model::FlowSchemaElement::builder()
            .resource_type("SAMPipelineTemplate")
            .name("SAM Pipeline Template")
            .description("Choose a template")
            .required(true)
            .build();
    
        let mut schema = HashMap::new();
        schema.insert("template_id".to_string(), template_step.clone());
    
        let manifests = Arc::clone(&self.manifests);
        let on_update = move |schema: &HashMap<String, toolkits::model::FlowSchemaElement>, state: &HashMap<String, serde_json::Value>| {
            let mut new_schema = HashMap::new();
            new_schema.insert("template_id".to_string(), schema.get("template_id").unwrap().clone());

            if let Some(template_id) = state.get("template_id").map(|x| x.as_str().expect("Template ID was not a string")) {
                if let Some(manifest) = manifests.get(template_id) {
                    let string_only_state: HashMap<String, String> = state.iter().map(|(k, v)| (k.to_string(), v.as_str().unwrap().to_string())).collect();
                    new_schema.extend(generate_schema(manifest, &string_only_state))
                }
            }

            new_schema
        };
    
        aws_flow::model::Flow::new(schema)
            .set_update_handler(Box::new(on_update))
    }
}

#[async_trait]
impl List<PipelineTemplate> for SamPipeline {
    type Error = Infallible;

    async fn list(&self) -> std::result::Result<Vec<Arc<PipelineTemplate>>, Self::Error> {
        Ok(self.templates.clone())
    }
}