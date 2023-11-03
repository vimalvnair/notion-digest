mod notion;

use std::{env, fs::File, io::Write};
use notion::NotionPage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let notion_links = get_notion_links().await?;
    save_notion_links_to_file(&notion_links);

    Ok(())
}

async fn get_notion_links() -> Result<Vec<notion::NotionLink>, Box<dyn std::error::Error>>{
    let mut request_params = notion::Params{
        sorts: vec![
            notion::Sort{
                property: "Created".to_owned(),
                direction: "descending".to_owned()
            }
        ],
        start_cursor: None
    };

    let mut notion_pages = notion::NotionPages{
        pages: Vec::new()
    };

    let mut notion_links: Vec<notion::NotionLink> = Vec::new();

    let mut has_more = true;

    while has_more{
        println!("Request params: {:?}", request_params);

        let response: NotionPage = get_notion_page(&request_params).await?;

        println!("first result : {:?}", response.results.get(0).unwrap());
        println!("cursor = {:?}", response.next_cursor);
        println!("result count = {}", response.results.len());


        has_more = response.has_more;

        if has_more{
            request_params.start_cursor = Some(response.next_cursor.clone().unwrap())
        }

        let links: Vec<notion::NotionLink> =  response.results.iter().map(|result| {
            let id = &result.id;
            let url = &result.properties.url.url;
            let title = &result.properties.name.title.get(0).unwrap().text.content;

            notion::NotionLink{
                id: id.to_string(),
                url: url.to_string(),
                title: title.to_string()
            }
        }).collect();

        notion_links.extend(links);

        notion_pages.pages.push(response);
    }

    Ok(notion_links)
}

async fn get_notion_page(request_params: &notion::Params) -> Result<NotionPage, Box<dyn std::error::Error>>{
    let notion_api_key = env::var("NOTION_API_KEY").unwrap();
    let notion_database_id = env::var("NOTION_DATABASE_ID").unwrap();

    let url = format!("https://api.notion.com/v1/databases/{notion_database_id}/query");

    println!("making request to: {}", url);

    let response: NotionPage = reqwest::Client::new()
        .post(&url)
        .json(&request_params)
        .bearer_auth(&notion_api_key)
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}

fn save_notion_links_to_file(notion_links: &Vec<notion::NotionLink>){
    let links = serde_json::to_string(notion_links).unwrap();
    let mut file = File::create("notion_links.json").unwrap();
    file.write_all(links.as_bytes()).unwrap();
}
