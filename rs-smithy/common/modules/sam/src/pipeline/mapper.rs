use std::collections::HashMap;

use toolkits::model::FlowSchemaElement;

use super::model::{QuestionKind, QuestionsManifest};


/// Builds all resolvable summaries 
type State = std::collections::HashMap<String, String>;
pub(super) fn generate_schema(manifest: &QuestionsManifest, state: &State) -> HashMap<String, FlowSchemaElement> {
    let mut schema = HashMap::new();

    // Skip all 'info' questions for now; later we'll need to do something else w/ em
    for q in manifest.list_with_state(state) {
        match q.kind {
            QuestionKind::Info => continue,
            _ => {
                if let Some(prompt) = manifest.resolve_expression(state, &q.question) {
                    let default_value = q.default.as_ref().map_or(None, |v| manifest.resolve_expression(state, &v));
                    let element = FlowSchemaElement::builder()
                        .description("Test")
                        .name(prompt)
                        .resource_type("string")
                        .set_valid_options(q.options.as_ref().map(|v| v.to_owned()))
                        .set_default_value(default_value)
                        .set_required(q.is_required)
                        .build();

                    schema.insert(q.key.clone(), element);
                } else {
                    // skip
                    continue
                }
            },
        }
    }

    schema
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summaries_test() {
        let questions = r#"{
        "questions": [
            {
                "key": "testing_stage_name",
                "question": "Select an index or enter the stage 1's configuration name (as provided during the bootstrapping)",
                "isRequired": true
            }, 
            {
                "key": "testing_stack_name",
                "question": "What is the sam application stack name for stage 1?",
                "isRequired": true,
                "default": "sam-app"
            }, 
            {
                "key": "testing_pipeline_execution_role",
                "question": "What is the pipeline execution role ARN for stage 1?",
                "isRequired": true,
                "allowAutofill": true,
                "default": {
                  "keyPath": [
                      { "valueOf": "testing_stage_name"},
                      "pipeline_execution_role"
                  ]
                }
            }
        ]
        }"#;

        let manifest = serde_json::de::from_str::<QuestionsManifest>(questions).expect("Invalid questions JSON");
        let mut state = std::collections::HashMap::new();
        state.insert("testing_stage_name".to_owned(), "foo".to_owned());
        state.insert("foo.pipeline_execution_role".to_owned(), "bar".to_owned());

        let schema = generate_schema(&manifest, &state);

        // BROKEN TESTS
        //assert_eq!(summaries.len(), 2);
        //assert_eq!(summaries[1].default_value.as_ref().expect("Default value missing"), "bar");
    }
}