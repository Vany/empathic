use std::env;
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub struct Config {
    pub root_dir: PathBuf,
    pub add_path: Vec<PathBuf>,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let root_dir = env::var("ROOT_DIR")
            .context("ROOT_DIR environment variable is required")?
            .into();
        
        let add_path = env::var("ADD_PATH")
            .unwrap_or_default()
            .split(':')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect();
        
        let log_level = env::var("LOGLEVEL")
            .unwrap_or_else(|_| "warn".to_string())
            .to_lowercase();
        
        Ok(Config {
            root_dir,
            add_path,
            log_level,
        })
    }

    pub fn project_path(&self, project: Option<&str>) -> PathBuf {
        match project {
            Some(project_name) => self.root_dir.join(project_name),
            None => self.root_dir.clone(),
        }
    }
}
