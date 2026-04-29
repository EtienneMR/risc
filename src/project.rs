use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct Project {
    includes: HashMap<String, PathBuf>,
    entrypoint: PathBuf,
}

impl Project {
    pub fn new(entrypoint: PathBuf) -> Self {
        let mut includes = HashMap::new();
        includes.insert(
            "@/".to_string(),
            entrypoint.parent().unwrap_or(&entrypoint).to_owned(),
        );
        Self {
            includes,
            entrypoint,
        }
    }

    pub fn entrypoint(&self) -> &PathBuf {
        &self.entrypoint
    }

    pub fn includes(&self) -> &HashMap<String, PathBuf> {
        &self.includes
    }

    pub fn resolve(root: &Path) -> Result<Self, Box<dyn Error>> {
        let file = root.join("risc.toml");

        match fs::read_to_string(&file) {
            Ok(source) => {
                let mut project: Project = toml::from_str(&source)?;

                if project.entrypoint.is_relative() {
                    project.entrypoint = root.join(&project.entrypoint);
                }

                Ok(project)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => match root.parent() {
                Some(parent) => Project::resolve(parent),
                None => Err(Box::new(e)),
            },
            Err(e) => Err(Box::new(e)),
        }
    }
}
