use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionPages{
    pub pages: Vec<NotionPage>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionLink{
    pub link_id: String,
    pub url: String,
    pub title: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionPage{
    pub results: Vec<NotionResult>,
    pub has_more: bool,
    pub next_cursor: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "Result"))]
pub struct NotionResult{
    #[serde(rename(deserialize = "id"))]
    pub link_id: String,
    pub properties: Properties
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Properties{
    #[serde(rename(deserialize = "URL"))]
    pub url: Url,
    #[serde(rename(deserialize = "Name"))]
    pub name: Name
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Url{
    pub url: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Name{
   pub title: Vec<Title>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Title{
    pub text: Content
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content{
    pub content: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Params{
    pub sorts: Vec<Sort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_cursor: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sort{
    pub property: String,
    pub direction: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SentLink{
    pub id: String,
    pub sent_at: String
}
