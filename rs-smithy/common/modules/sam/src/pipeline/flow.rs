use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::thread::JoinHandle;

use tokio::fs::{read, create_dir};
use tokio::process::Command;
use super::model::{Manifest, QuestionsManifest, Template};

const TEMPLATE_REPO_URL: &str = "https://github.com/aws/aws-sam-cli-pipeline-init-templates.git";


type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) struct FlowContext {
    pub state: HashMap<String, String>,
    // We'll keep cloning it for now, though this struct shouldn't own it
    pub questions_manifest: QuestionsManifest,
    // just add state here?
    // TODO: cookiecutter
    // TODO: manifest
    // TODO: metadata
}

// Util struct

/// Temporary directory. Automatically cleaned-up on drop.

pub(super) struct TempDir {
    path: PathBuf
}

impl TempDir {
    async fn create() -> Result<Self> {
        let mut path = std::env::temp_dir();

        path.push("toolkit_tmp_dir".to_owned());
        create_dir(&path).await?;

        Ok(TempDir { path })
    }

    fn get<'a>(&'a self) -> &'a PathBuf {
        &self.path
    }
}

impl AsRef<PathBuf> for TempDir {
    fn as_ref(&self) -> &PathBuf {
        &self.path
    }
}

impl Deref for TempDir {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Is unwrapping dangerous here??
        let path = self.path.to_str().unwrap().to_owned();

        // THIS BLOCKS THE THREAD!
        match std::fs::remove_dir_all(path) {
            Ok(_) => {},
            Err(err) => println!("{:?}", err),
        };
    }
}

//

/// Clones the template repository into a temp folder
pub(super) async fn clone_templates() -> Result<TempDir> {
    //let tmp_dir = tmp_path.to_str().to_owned().map_or(Err("Bad temp dir"), Ok)?;
    let tmp = TempDir::create().await?;
    //let tmp_dir = tmp.get().to_str().to_owned().map_or(Err("Bad temp dir"), Ok)?;

    let git_clone = Command::new("git")
        .arg("clone")
        .arg(TEMPLATE_REPO_URL)
        .arg(tmp.as_ref())
        .output()
        .await;

    match git_clone {
        Ok(_output) => Ok(tmp),
        Err(_err) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Git clone failed"))),
    }
}

// We can probably just drop the 'questions manifest' since the format doesn't work too well for our use-case
// Plus we'd want to change a bunch of stuff anyway so better to not use it all
pub(super) async fn parse_context(template_dir: &PathBuf) -> Result<FlowContext> {
    let questions_path = template_dir.join("questions.json");
    let contents = read(questions_path).await?;
    let utf8 = std::str::from_utf8(&contents).map_or(Err("Unable to read questions file"), Ok)?;
    let questions = serde_json::de::from_str::<QuestionsManifest>(utf8)?;

    Ok(FlowContext { questions_manifest: questions, state: HashMap::new() })
}


pub(super) async fn list_templates(repo_root: &PathBuf) -> Result<Vec<Template>> {
    let manifest_path = repo_root.join("manifest.yaml");
    let contents = read(manifest_path).await?;
    let utf8 = std::str::from_utf8(&contents).map_or(Err("Unable to read template manifest"), Ok)?;
    let manifest = serde_yaml::from_str::<Manifest>(utf8)?;

    Ok(manifest.templates)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn happy_path() -> Result<()> {
        let repo = clone_templates().await?;
        assert!(repo.as_ref().ends_with("/toolkit_tmp_dir"));
        drop(repo);
        Ok(())
    }

    #[tokio::test]
    async fn parse_questions() -> Result<()> {
        // TODO: move to `resources/test`
        let fixtures = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/pipeline/fixtures");
        let context = parse_context(&fixtures).await?;

        assert!(context.questions_manifest.list().len() > 0);

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn integ_test() -> Result<()> {
        let target = "Jenkins/two-stage-pipeline-template";
        let repo = clone_templates().await?;
        let context = parse_context(&(repo.as_ref().join(target))).await?;

        assert!(context.questions_manifest.list().len() > 0);

        drop(repo);
        Ok(())
    }
}