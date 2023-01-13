use anyhow::{Context, Result};
use std::{collections::HashMap};
use serde::{Serialize, Deserialize};
use crate::{ExtraArgs, LoadConfig, Validate, utils};
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

    #[serde(default, skip_serializing_if= "is_default")]
    pub res: ResponseProfile
}

pub fn is_default<T: Default + PartialEq>(x : &T) -> bool {
    x == &T::default()
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    pub skip_headers: Vec<String>,

    #[serde(skip_serializing_if="Vec::is_empty", default)]
    pub skip_body: Vec<String>
}

impl ResponseProfile {
    pub fn new(skip_headers: Vec<String>, skip_body: Vec<String>) -> Self {
        Self {
            skip_headers,
            skip_body
        }
    }
}

impl DiffConfig {
    pub fn new(map: HashMap<String, DiffProfile>) -> Self {
        Self {
            profiles: map
        }
    }

    pub fn get_profile(&self, name:&str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

impl Validate for DiffConfig {
    fn validate(&self) -> Result<()> {
        for (name, profile) in &self.profiles {
            profile.validate().context(format!("Parse profile {} occur a error.", name))?;
        }
        Ok(())
    }
}

impl LoadConfig for DiffConfig {}

impl DiffProfile {
    pub fn new(req1: RequestProfile, req2: RequestProfile, res: ResponseProfile) -> Self {
        Self {
            req1,
            req2,
            res
        }
    }

    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let res1 = self.req1.send(&args).await?;
        let res2 = self.req1.send(&args).await?;

        let text1 = res1.filter_text(&self.res).await?;
        let text2 = res2.filter_text(&self.res).await?;

        let result = utils::diff_text(text1, text2)?;

        Ok(result)
    }

    pub fn validate(&self) ->Result<()> {
        self.req1.validate().context("Parse req1 occur a error.")?;
        self.req2.validate().context("Parse req2 occur a error.")?;
        Ok(())
    }
}