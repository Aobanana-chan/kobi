pub use super::types::*;
use crate::copy_client::{
    ComicChapter, ComicData, ComicInSearch, ComicQuery, Page, RankItem, Response, Tags,
};
use reqwest::Method;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Client {
    agent: Mutex<Arc<reqwest::Client>>,
    api_host: Mutex<Arc<String>>,
}

impl Client {
    pub fn new(agent: impl Into<Arc<reqwest::Client>>, api_host: impl Into<String>) -> Self {
        Self {
            agent: Mutex::new(agent.into()),
            api_host: Mutex::new(Arc::new(api_host.into())),
        }
    }

    pub async fn set_agent(&self, agent: impl Into<Arc<reqwest::Client>>) {
        let mut lock = self.agent.lock().await;
        *lock = agent.into();
    }

    pub async fn set_api_host(&self, api_host: impl Into<String>) {
        let mut lock = self.api_host.lock().await;
        *lock = Arc::new(api_host.into());
    }

    pub async fn api_host_string(&self) -> Arc<String> {
        let api_host = self.api_host.lock().await;
        api_host.clone()
    }

    pub async fn request<T: for<'de> serde::Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let obj = params.as_object().expect("query must be object");
        let agent_lock = self.agent.lock().await;
        let agent = agent_lock.clone();
        drop(agent_lock);
        let request = agent.request(
            method.clone(),
            format!("{}{}", &self.api_host_string().await.as_str(), path),
        );
        let request = request
            .header("authorization", "Token")
            .header("referer", "com.copymanga.app-2.0.7")
            .header("User-Agent", "COPY/2.0.7")
            .header("source", "copyApp")
            .header("webp", "1")
            .header("version", "2.0.7")
            .header("region", "1")
            .header("platform", "3")
            .header("Accept", "application/json");
        let request = match method {
            reqwest::Method::GET => request.query(&obj),
            _ => request.form(&obj),
        };
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        if status.as_u16() == 404 {
            return Err(Error::message("404 Not found"));
        }
        println!("{} {}", status, text);
        let response: Response = serde_json::from_str(text.as_str())?;
        if response.code != 200 {
            return Err(Error::message(response.message));
        }
        Ok(serde_json::from_value(response.results)?)
    }

    pub async fn tags(&self) -> Result<Tags> {
        self.request(
            reqwest::Method::GET,
            "/api/v3/h5/filter/comic/tags",
            serde_json::json!({
                "platform": 3,
            }),
        )
        .await
    }

    pub async fn comic_search(
        &self,
        q_type: &str,
        q: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Page<ComicInSearch>> {
        self.request(
            reqwest::Method::GET,
            "/api/v3/search/comic",
            serde_json::json!({
                "platform": 3,
                "limit": limit,
                "offset": offset,
                "q": q,
                "q_type": q_type,
            }),
        )
        .await
    }

    pub async fn comic_rank(
        &self,
        date_type: &str,
        offset: u64,
        limit: u64,
    ) -> Result<Page<RankItem>> {
        self.request(
            reqwest::Method::GET,
            "/api/v3/ranks",
            serde_json::json!({
                "platform": 3,
                "date_type": date_type,
                "offset": offset,
                "limit": limit,
            }),
        )
        .await
    }

    pub async fn comic(&self, path_word: &str) -> Result<ComicData> {
        self.request(
            reqwest::Method::GET,
            format!("/api/v3/comic2/{path_word}").as_str(),
            serde_json::json!({
                 "platform": 3,
            }),
        )
        .await
    }

    pub async fn comic_chapter(
        &self,
        comic_path_word: &str,
        group_path_word: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Page<ComicChapter>> {
        self.request(
            reqwest::Method::GET,
            format!("/api/v3/comic/{comic_path_word}/group/{group_path_word}/chapters").as_str(),
            serde_json::json!({
                "offset": offset,
                "limit": limit,
                "platform": 3,
            }),
        )
        .await
    }

    pub async fn comic_query(&self, path_word: &str) -> Result<ComicQuery> {
        self.request(
            reqwest::Method::GET,
            format!("/api/v3/comic2/{path_word}/query ").as_str(),
            serde_json::json!({
                 "platform": 3,
            }),
        )
        .await
    }

    pub async fn download_image(&self, url: &str) -> Result<bytes::Bytes> {
        let agent_lock = self.agent.lock().await;
        let agent = agent_lock.clone();
        drop(agent_lock);
        Ok(agent.get(url).send().await?.bytes().await?)
    }
}