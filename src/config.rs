use anyhow::Result;
use std::{collections::HashMap};
use serde::{Serialize, Deserialize};
use tokio::fs;
use crate::{ExtraArgs, utils};
use crate::req::RequestProfile;

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
    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let res1 = self.req1.send(&args).await?;
        let res2 = self.req1.send(&args).await?;

        let text1 = res1.filter_text(&self.res).await?;
        let text2 = res2.filter_text(&self.res).await?;

        let result = utils::diff_text(text1, text2)?;

        Ok(result)
    }
}