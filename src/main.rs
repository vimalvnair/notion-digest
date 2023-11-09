use std::env;
use std::path::Path;
use std::fs::{File, self, OpenOptions};
use std::io::{Write, Read, Seek};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

mod notion;
mod sendgrid;
use notion::NotionPage;

const NOTION_LINKS_FILENAME: &str = "notion_links.json";
const SENT_NOTION_LINKS_FILENAME: &str = "sent_notion_links.json";
const NUMBER_OF_LINKS_TO_FECTH: usize = 3;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let notion_links_file = Path::new(NOTION_LINKS_FILENAME);

    if notion_links_file.exists() {
        println!("links already saved! no need to fetch");

        // sendgrid::send_email().await?;
        let mut stored_link_ids: Vec<String> = Vec::new();

        let stored_links_path = Path::new(SENT_NOTION_LINKS_FILENAME);

        if stored_links_path.exists() {
            let stored_links_content = fs::read_to_string(SENT_NOTION_LINKS_FILENAME).unwrap();
            let parsed_stored_links: Vec<notion::SentLink> = serde_json::from_str(&stored_links_content).unwrap();
            stored_link_ids = parsed_stored_links.iter().map(|link| link.id.to_string() ).collect();
        }

        let notion_links = fs::read_to_string(NOTION_LINKS_FILENAME).unwrap();
        let notion_links: Vec<notion::NotionLink> = serde_json::from_str(&notion_links).unwrap();
        let notion_links: Vec<notion::NotionLink> = notion_links.into_iter().filter(|link|{
            !stored_link_ids.contains(&link.link_id)
        }).collect();

        let total_number_of_links = notion_links.len();
        let number_of_links_to_fetch = if total_number_of_links > NUMBER_OF_LINKS_TO_FECTH { NUMBER_OF_LINKS_TO_FECTH } else { total_number_of_links };
        println!("total_number_of_links = {}", total_number_of_links);
        let mut rng = rand::thread_rng();
        let links_to_send: Vec<&notion::NotionLink> = (0..number_of_links_to_fetch).into_iter()
            .map(|_|{
                let random_index = rng.gen_range(0..total_number_of_links);

                println!("Fetching link at index: {}", random_index);

                let random_notion_link = notion_links.get(random_index).unwrap();

                println!("Random notion link = {:?}", random_notion_link);

                random_notion_link
            }).collect();

        println!("links to send = {:?}", links_to_send);

        record_sent_links(&links_to_send);
    } else {
        println!("getting links from notion");

        let notion_links = get_notion_links().await?;
        save_notion_links_to_file(&notion_links);
    }

    Ok(())
}

fn record_sent_links(sent_links: &Vec<&notion::NotionLink>){
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(SENT_NOTION_LINKS_FILENAME)
        .unwrap();

    let mut sent_links: Vec<notion::SentLink> = sent_links.iter().map(|link| {
        notion::SentLink{
            id: link.link_id.to_string(),
            sent_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()
        }
    }).collect();

    if file.metadata().unwrap().len() == 0 {
        file.write_all(serde_json::to_string(&sent_links).unwrap().as_bytes()).unwrap();
    } else {
        let mut stored_links = String::new();
        file.read_to_string(&mut stored_links).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();

        let stored_links: Vec<notion::SentLink> = serde_json::from_str(&stored_links).unwrap();

        sent_links.extend(stored_links);
        println!("sent_links_count: {}", sent_links.len());
        file.write_all(serde_json::to_string(&sent_links).unwrap().as_bytes()).unwrap();
    }
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
            let id = &result.link_id;
            let url = &result.properties.url.url;
            let title = &result.properties.name.title.get(0).unwrap().text.content;

            notion::NotionLink{
                link_id: id.to_string(),
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
    let mut file = File::create(NOTION_LINKS_FILENAME).unwrap();
    file.write_all(links.as_bytes()).unwrap();
}
