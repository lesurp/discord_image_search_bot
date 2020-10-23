use crate::ImageBotError;
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde_json::Value;

#[async_trait]
pub trait ImageSearcher {
    async fn search(&self, query: &str) -> Result<String, ImageBotError>;
}

pub struct GoogleImageSeacher {
    api_key: String,
    cx_id: String,
}

impl GoogleImageSeacher {
    pub fn new<S: Into<String>, SS: Into<String>>(api_key: S, cx_id: SS) -> Self {
        Self {
            api_key: api_key.into(),
            cx_id: cx_id.into(),
        }
    }
}

#[async_trait]
impl ImageSearcher for GoogleImageSeacher {
    async fn search(&self, query: &str) -> Result<String, ImageBotError> {
        let params = [
            ("start", "1"),
            ("num", "1"),
            ("q", query),
            ("imgSize", "medium"),
            ("key", &self.api_key),
            ("cx", &self.cx_id),
        ];

        let client = reqwest::Client::new();
        let req = client
            .get("https://www.googleapis.com/customsearch/v1")
            .query(&params);
        let out = req.send().await.unwrap().json::<Value>().await.unwrap();

        match &out["items"][0]["pagemap"]["cse_thumbnail"][0]["src"] {
            Value::String(v) => Ok(v.clone()),
            not_a_str => {
                return Err(ImageBotError::Api(
                    format!("Expected a string, got a '{}'", not_a_str).to_owned(),
                ))
            }
        }
    }
}

pub struct RapidApiImageSeacher {
    api_key: String,
}

impl RapidApiImageSeacher {
    pub fn new<S: Into<String>>(api_key: S) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
}

#[async_trait]
impl ImageSearcher for RapidApiImageSeacher {
    async fn search(&self, query: &str) -> Result<String, ImageBotError> {
        let params = [
            ("pageNumber", "1"),
            ("pageSize", "1"),
            ("q", query),
            ("autoCorrect", "true"),
            ("safeSearch", "false"),
        ];

        let mut headers = HeaderMap::new();

        headers.insert(
            "x-rapidapi-host",
            "contextualwebsearch-websearch-v1.p.rapidapi.com"
                .parse()
                .unwrap(),
        );
        headers.insert("x-rapidapi-key", self.api_key.parse().unwrap());

        let client = reqwest::Client::new();
        let req = client
            .get("https://rapidapi.p.rapidapi.com/api/Search/ImageSearchAPI")
            .query(&params)
            .headers(headers);
        let out = req.send().await?.json::<Value>().await?;

        match &out["value"][0]["url"] {
            Value::String(v) => Ok(v.clone()),
            not_a_str => {
                return Err(ImageBotError::Api(
                    format!("Expected a string, got a '{}'", not_a_str).to_owned(),
                ))
            }
        }
    }
}
