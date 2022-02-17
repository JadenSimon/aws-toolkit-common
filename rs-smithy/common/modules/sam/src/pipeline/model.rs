use aws_core::resource::Resource;
use serde::Deserialize;
use serde_json::Map;

// imports unrelated to deserialization
use std::collections::HashMap;

// https://github.com/aws/aws-sam-cli-pipeline-init-templates.git
// We need to do some light parsing here
// Note that this is all related to `sam pipeline init` -not- `sam pipeline bootstrap`
// The `init` command is analagous to `sam init`

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub(super) enum KeyPathElement {
    #[serde(rename_all = "camelCase")]
    Ref { value_of: String },
    Literal(String),
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub(super) enum QuestionExpression {
    /// Looks like some sort of reference-like object that points to something which may not necessarily 
    /// exist within the schema itself.
    /// 
    /// This should be resolved by joining all key path elements together, then looking it up in the document.
    /// If it isn't found, then we need to defer to hand-written code.
    #[serde(rename_all = "camelCase")]
    Ref { key_path: Vec<KeyPathElement> },
    Literal(String),
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")] 
pub(super) enum QuestionKind {
    Info,
    Choice,
    Confirm,
    Question,
}

impl Default for QuestionKind {
    fn default() -> Self { Self::Question }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct Question {
    /// The type of question. Seems like by default it's a prompt, otherwise it's a message ('info', 'confirm', etc.)
    #[serde(default)]
    pub kind: QuestionKind,

    /// Maps to parameter to the pipeline.toml file
    pub key: String,
    /// User-facing prompt
    pub question: QuestionExpression, 
    /// Default value if not answering the question
    pub default: Option<QuestionExpression>,

    /// Restricts the selection 
    pub options: Option<Vec<String>>,

    /// Looks to be a map that describes what the next question should be given a certain response
    /// 
    /// Example:
    /// ```json
    /// {
    ///   "options": ["Bitbucket", "CodeCommit", "GitHub", "GitHubEnterpriseServer"],
    ///   "nextQuestion": {  
    ///     "CodeCommit": "codecommit_repository_name"  
    ///    }
    /// }
    /// ```  
    /// 
    pub next_question: Option<serde_json::Value>,

    /// Next question to use if `next_question` is not applicable
    pub default_next_question: Option<String>,

    /// This is false by default (I think))
    pub is_required: Option<bool>,

    /// Presumably pre-populates the prompt with a default
    pub allow_autofill: Option<bool>,
}

// There appears to be special 'keys' that tell SAM CLI to do something:
// 'stage_names_message' --> list all config names in `pipelineconfig.toml`
// from https://github.com/aws/aws-sam-cli/blob/fe612397fb946baaa2dc9557ac439f45cfec36ba/samcli/commands/pipeline/init/interactive_init_flow.py#L251
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TemplateProvider {
    pub id: String,
    pub display_name: String,
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Metadata {
    /// Number of stages in the pipeline template
    number_of_stages: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Template {
    pub provider: String, // ID of TemplateProvider
    pub location: String, // Relative path of repo root
    pub display_name: String,
}

// why does it have to be YAML
// we can ignore the manifest for now; it is only important when 'compiling' the answers
#[derive(Deserialize)]
pub(super) struct Manifest {
    pub providers: Vec<TemplateProvider>,
    pub templates: Vec<Template>, // temp visibility
}

// found at `questions.json`
#[derive(Deserialize, Clone)]
pub(super) struct QuestionsManifest {
    questions: Vec<Question>,
}

impl QuestionsManifest {
    pub fn list(&self) -> &Vec<Question> {
        &self.questions
    }

    // TODO: map Question to something nicer?
    /// Lists all questions filtered by the currently assigned keys
    pub fn list_with_state<'a>(&'a self, state: &HashMap<String, String>) -> Vec<&'a Question> {
        let mut results = Vec::new();

        let q = self.questions.iter().filter(|x| !state.contains_key(&x.key));
        results.extend(q);

        results
    }

    /// This resolves an expression into the desired key path  \
    /// Probably want to get from the map??
    pub fn resolve_expression(&self, state: &HashMap<String, String>, expression: &QuestionExpression) -> Option<String> {
        let key_path = match expression {
            QuestionExpression::Ref { key_path } => key_path,
            QuestionExpression::Literal(s) => return Some(s.to_owned()),
        };

        let mut result = Vec::new();
        
        for element in key_path {
            let resolved = match element {
                KeyPathElement::Ref { value_of } => {
                    match state.get(value_of) {
                        Some(v) => v,
                        _ => return None,
                    }
                }
                KeyPathElement::Literal(s) => s,
            };

            result.push(resolved.to_owned());
        }

        state.get(&result.join(".")).map(|x| x.to_owned())
    }

    // The dependency graph will probably need to be hand-curated
}

/// Appears to represent the initial state of the interactive flow, containing key/value pairs of which most are empty
/// 
/// Located at `cookiecutter.json`
pub type Cookiecutter = Map<String, String>;

/// Do not confuse these with CFN templates. These have nothing to do with CFN.
pub struct PipelineTemplate {
    /// Unique ID of the template
    id: String,

    /// User-friendly name of the template
    name: String,

    /// Optional description
    description: Option<String>,
}

impl PipelineTemplate {
    pub fn new(id: String, name: String, description: Option<String>) -> Self {
        Self { id, name, description }
    }
}

impl From<Template> for PipelineTemplate {
    fn from(template: Template) -> Self {
        Self::new(template.provider, template.display_name, Some("Example".to_string()))
    }
}

impl Resource for PipelineTemplate {
    fn summary(&self) -> toolkits::model::ResourceSummary {
        toolkits::model::ResourceSummary::builder()
            .name(&self.name)
            .iri("aws:sam/pipeline/template/".to_owned() + &self.id)
            .set_description(self.description.clone())
            .resource_type("SAMPipelineTemplate")
            .build()
    }
}


// This code is what does the final generation: https://github.com/aws/aws-sam-cli/blob/fe612397fb946baaa2dc9557ac439f45cfec36ba/samcli/commands/pipeline/init/interactive_init_flow.py#L195
