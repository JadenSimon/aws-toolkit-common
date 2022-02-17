use yaml_rust::{Yaml, YamlLoader, YamlEmitter};

const SERVERLESS_API: &str = "AWS::Serverless::Api";
const SERVERLESS_FUNCTION: &str = "AWS::Serverless::Function";
const LAMBDA_FUNCTION: &str = "AWS::Lambda::Function";

pub type YamlMap = yaml_rust::yaml::Hash;

// Just gets serverless/lambda resource types?
pub fn get_lambda_resources(yaml: &Yaml) -> Vec<&YamlMap> {
    let mut found = Vec::new();
    
    if let Yaml::Hash(resources) = &yaml["Resources"] {
        for resource in resources.values() {
            match &resource["Type"] {
                Yaml::String(val) if val.eq(LAMBDA_FUNCTION) || val.eq(SERVERLESS_FUNCTION) => {
                    found.push(resource.as_hash().unwrap());
                },
                _ => {},
            }
        }
    }

    found
}

// add helper traits for type-safety?
// probably easier to just codegen bindings

