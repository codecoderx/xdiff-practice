pub mod config;
pub mod cli;
pub mod req;
pub mod utils;

use cli::{KeyVal, KeyValType};

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