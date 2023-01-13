use serde::{Serialize, Deserialize};
use std::{fmt::Debug, str::FromStr};
use reqwest::{Method, Response, Client};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use url::Url;
use std::fmt::Write;

use crate::{ExtraArgs, Validate, config::ResponseProfile};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestProfile {
    #[serde(with="http_serde::method", default)]
    pub method: Method,
    pub url: Url,

    #[serde(skip_serializing_if="HeaderMap::is_empty", with= "http_serde::header_map", default)]
    pub headers: HeaderMap,

    #[serde(skip_serializing_if="not_json_object", default)]
    pub params: Option<Value>,

    #[serde(skip_serializing_if="not_json_object", default)]
    pub body: Option<Value>
}

pub fn not_json_object(v: &Option<Value>) -> bool {
    v.as_ref().map_or(true, |x| x.is_null() || (x.is_object() && x.as_object().unwrap().is_empty()) || !x.is_object())
}

pub struct ResponseExt(Response);

impl RequestProfile {
    pub fn new(method: Method, url: Url, headers: HeaderMap, params: Option<Value>, body: Option<Value>) -> Self {
        Self {
            method, 
            url, 
            headers,
            params, 
            body
        }
    }

    pub async fn send(&self, args: &ExtraArgs)-> Result<ResponseExt> {
        let client = Client::new();
        let (headers, query, body) = self.generate(args)?;
        let request = client.request(self.method.clone(), self.url.clone())
            .headers(headers)
            .query(&query)
            .body(body)
            .build()?;

        let response = client.execute(request).await?;
        Ok(ResponseExt(response))
    }

    pub fn generate(&self, args: &ExtraArgs) -> Result<(HeaderMap, Value, String)> {
        let mut headers = self.headers.clone();
        let mut query  = self.params.clone().unwrap_or_else(|| json!({}));
        let mut body  = self.body.clone().unwrap_or_else(|| json!({}));

        for (k,v) in &args.headers {
            headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
        }

        if !headers.contains_key(CONTENT_TYPE) {
            headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
        }

        for (k, v) in &args.params {
            query[k] = v.parse()?;
        }

        for (k, v) in &args.body {
            body[k] = v.parse()?;
        }

        let content_type = get_content_type(&headers)?;
        let body = match content_type.as_deref() {
            Some("application/json") => serde_json::to_string(&body)?,
            Some("application/x-www-form-urlencoded"| "multipart/form-data") => serde_urlencoded::to_string(&body)?,
            _=> return Err(anyhow::anyhow!("Unsupport content-type"))
        };

        Ok((headers, query, body))
    }
}

impl Validate for RequestProfile {
    fn validate(&self)-> Result<()> {
        if let Some(body) = self.body.as_ref() {
            if !body.is_object() {
                return Err(anyhow::anyhow!("Parse body is not a object. \n{}\n", serde_yaml::to_string(body)?))
            }   
        }

        if let Some(params) = self.params.as_ref() {
            if !params.is_object() {
                return Err(anyhow::anyhow!("Parse params is not a object. \n{}\n", serde_yaml::to_string(params)?))
            }   
        }

        Ok(())
    }
}

impl FromStr for RequestProfile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let url = Url::parse(s)?;
        let query = url.query_pairs();
        let mut params = json!({});
        for (k, v) in query {
            params[&*k] = v.parse().context(format!("Parse {}:{} fail.", &*k, v))?;
        }

        Ok(RequestProfile::new(
            Method::GET,
            url,
            HeaderMap::default(),
            Some(params),
            None
        ))
    }
}



impl ResponseExt {
    pub async fn filter_text(self, profile: &ResponseProfile)-> Result<String> {
        let mut output = String::new();
        writeln!(&mut output,"{:?} {}",self.0.version(), self.0.status())?;
        let headers = self.0.headers() ;
        for header in headers {
            if !profile.skip_headers.contains(&String::from_str(header.0.as_str())?) {
                writeln!(&mut output, "{}: {}", header.0, header.1.to_str()?)?;
            }
        }

        let content_type = get_content_type(&headers)?;
        let body = self.0.text().await?;
        match content_type.as_deref() {
            Some("application/json") => {
                let mut body: Value = serde_json::from_str(&body)?;
                match body {
                    Value::Object(ref mut map) => {
                        for skip in &profile.skip_body {
                            if map.contains_key(skip) {
                                map.remove(skip);
                            }
                        }
                    }
                    _=> body = json!({})
                }
                write!(&mut output,"{}", body.to_string())?;
            },
            _ => write!(&mut output,"{}", body.to_string())?,
        };

        Ok(output.to_string())
    }

    pub fn get_header_keys(&self) -> Result<Vec<String>> {
        Ok(self.0.headers().iter().map(|(key,_)| key.to_string()).collect())
    }
}

fn get_content_type(headers: &HeaderMap) -> Result<Option<String>> {
    Ok(headers.get(CONTENT_TYPE).and_then(|v| v.to_str().unwrap().split(";").next().map(|k| k.to_string())))
}