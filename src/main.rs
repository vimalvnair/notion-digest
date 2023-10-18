use std::env;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct NotionPage{
    results: Vec<NotionResult>,
    has_more: bool,
    next_cursor: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "Result"))]
struct NotionResult{
    id: String,
    properties: Properties
}

#[derive(Serialize, Deserialize, Debug)]
struct Properties{
    #[serde(rename(deserialize = "URL"))]
    url: Url,
    #[serde(rename(deserialize = "Name"))]
    name: Name
}

#[derive(Serialize, Deserialize, Debug)]
struct Url{
    url: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Name{
    title: Vec<Title>
}

#[derive(Serialize, Deserialize, Debug)]
struct Title{
    text: Content
}

#[derive(Serialize, Deserialize, Debug)]
struct Content{
    content: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let notion_api_key = env::var("NOTION_API_KEY").unwrap();
    let notion_database_id = env::var("NOTION_DATABASE_ID").unwrap();

    let url = format!("https://api.notion.com/v1/databases/{notion_database_id}/query");

    let response: NotionPage = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&notion_api_key)
        .send()
        .await?
        .json()
        .await?;

    println!("making request to: {}", url);
    println!("first result : {:?}", response.results.get(0).unwrap());
    println!("cursor = {:?}", response.next_cursor);
    println!("result count = {}", response.results.len());

    Ok(())
}
