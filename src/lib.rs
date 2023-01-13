pub mod config;
pub mod cli;
pub mod req;
pub mod utils;

use cli::{KeyVal, KeyValType};
use serde::de::DeserializeOwned;
use tokio::fs;
use anyhow::Result;
use async_trait::async_trait;

pub trait Validate {
    fn validate(&self) -> Result<()>;
}

#[async_trait]
pub trait LoadConfig: Sized + DeserializeOwned + Validate {
    async fn load_yaml(path: &str) ->  Result<Self> {
        let data = fs::read_to_string(path).await?;
        Self::from_yaml(&data)
    }

    fn from_yaml(data: &str) -> Result<Self>  {
        let config: Self = serde_yaml::from_str(data)?;
        config.validate()?;
        Ok(config)
    }
}

#[derive(Debug, Default)]
pub struct ExtraArgs {
    pub headers: Vec<(String,String)>,
    pub body: Vec<(String,String)>,
    pub params: Vec<(String,String)>
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(data: Vec<KeyVal>) -> ExtraArgs {
        let mut headers = Vec::new();
        let mut body = Vec::new();
        let mut params = Vec::new();

        if !data.is_empty() {
            for key_val in  data{
                match key_val.key_type {
                    KeyValType::Header => headers.push((key_val.key, key_val.val)),
                    KeyValType::Body => body.push((key_val.key, key_val.val)),
                    KeyValType::Query => params.push((key_val.key, key_val.val))
                }
            }
        }

        ExtraArgs{
            headers,
            body,
            params
        }
    }
}