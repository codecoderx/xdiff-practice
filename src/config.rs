use anyhow::Result;
use serde_yaml::Value;
use std::{collections::HashMap};
use serde::{Serialize, Deserialize};
use reqwest::{Method, header::HeaderMap};
use url::Url;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    pub res: ResponseProfile
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestProfile {
    #[serde(with="http_serde::method", default)]
    pub method: Method,
    pub url: Url,

    #[serde(skip_serializing_if="HeaderMap::is_empty", with= "http_serde::header_map", default)]
    pub headers: HeaderMap,

    #[serde(skip_serializing_if="Option::is_none", default)]
    pub params: Option<Value>,

    #[serde(skip_serializing_if="Option::is_none", default)]
    pub body: Option<Value>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    pub skip_headers: Vec<String>,

    #[serde(skip_serializing_if="Vec::is_empty", default)]
    pub skip_body: Vec<String>
}

impl DiffConfig {

    pub async fn load_yaml(path: &str) ->  Result<Self> {
        let data = fs::read_to_string(path).await?;
        Self::from_yaml(&data)
    }

    pub fn from_yaml(data: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(data)?)
    }

    pub fn get_profile(&self, name:&str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

impl DiffProfile {
    
}

#[derive(Debug)]
pub struct DiffArgs {
    
}