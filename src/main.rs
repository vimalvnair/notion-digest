use std::env;
use std::path::Path;
use std::fs::{File, self, OpenOptions};
use std::io::{Write, Read, Seek};
use std::time::{SystemTime, UNIX_EPOCH};
use askama::Template;
use futures::future;
use rand::seq::SliceRandom;

use notion::NotionPage;
use scraper::{Html, Selector};

mod notion;
mod sendgrid;
mod local_mail;

const NOTION_LINKS_FILENAME: &str = "notion_links.json";
const SENT_NOTION_LINKS_FILENAME: &str = "sent_notion_links.json";
const NUMBER_OF_LINKS_TO_FECTH: usize = 3;

#[derive(Template)]
#[template(path="email.html")]
struct EmailTemplate<'a>{
    links: Vec<EmailLink<'a>>
}

struct EmailLink<'a>{
    title: &'a String,
    url: &'a String,
    description: String,
    image_url: String
}

impl<'a> EmailLink<'a> {
    pub async fn new(link: &'a notion::NotionLink) -> Result<Self, Box<dyn std::error::Error>>{
        let og_attributes = Self::get_description_and_image_url(&link.url).await?;

        Ok(EmailLink{
            title: &link.title,
            url: &link.url,
            description: og_attributes.0,
            image_url: og_attributes.1
        })
    }

    async fn get_description_and_image_url(url: &String) -> Result<(String, String), Box<dyn std::error::Error>>{
        let body = reqwest::get(url).await?.text().await?;
        let html_fragment = Html::parse_document(&body);

        let description = Self::get_og_attribute(&html_fragment, "description");
        let image = Self::get_og_attribute(&html_fragment, "image");

        println!("description = {:?}", description);

        Ok((description, image))
    }

    fn get_og_attribute(html_fragment: &Html, og_attribute: &str) -> String{
        let og_description_selector = Selector::parse(&format!("meta[property='og:{}']", og_attribute)).unwrap();

        match html_fragment.select(&og_description_selector).next(){
            Some(element) => if let Some(content) = element.value().attr("content"){
                content.to_string()
            } else {
                String::new()
            },
            None => String::new()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let send_email_via_sendgrid = env::var("USE_SENDGRID").is_ok();
    let notion_links = notion_links().await?;
    let mut stored_link_ids: Vec<String> = Vec::new();
    let stored_links_path = Path::new(SENT_NOTION_LINKS_FILENAME);

    if stored_links_path.exists() {
        let stored_links_content = fs::read_to_string(SENT_NOTION_LINKS_FILENAME).unwrap();
        let parsed_stored_links: Vec<notion::SentLink> = serde_json::from_str(&stored_links_content).unwrap();
        stored_link_ids = parsed_stored_links.iter().map(|link| link.id.to_string() ).collect();
    }

    let filtered_notion_links: Vec<notion::NotionLink> = notion_links.into_iter().filter(|link|{
        !stored_link_ids.contains(&link.link_id)
    }).collect();

    let total_number_of_links = filtered_notion_links.len();
    let number_of_links_to_fetch = if total_number_of_links > NUMBER_OF_LINKS_TO_FECTH { NUMBER_OF_LINKS_TO_FECTH } else { total_number_of_links };
    println!("total_number_of_links = {}", total_number_of_links);

    let mut rng = rand::thread_rng();
    let mut numbers: Vec<usize> = (0..total_number_of_links).into_iter().collect();
    numbers.shuffle(&mut rng);
    let random_indices = &numbers[0..number_of_links_to_fetch];
    println!("shuffled random numbers: {:?}", random_indices);

    let links_to_send: Vec<&notion::NotionLink> = random_indices.into_iter()
    .map(|idx|{
        println!("Fetching link at index: {}", idx);

        let random_notion_link = filtered_notion_links.get(*idx).unwrap();

        println!("Random notion link = {:?}", random_notion_link);

        random_notion_link
    }).collect();

    println!("links to send = {:?}", links_to_send);
    record_sent_links(&links_to_send);

    let template = EmailTemplate{
        links: build_email_body(links_to_send).await
    };

    let email_body = template.render().unwrap();

    if send_email_via_sendgrid {
        sendgrid::send_email(email_body).await?;
    } else {
        local_mail::send_local_mail(email_body);
    }

    Ok(())
}

async fn build_email_body(links: Vec<&notion::NotionLink>) -> Vec<EmailLink>{
    let email_futures: Vec<_> = links.into_iter().map(|link|{
        EmailLink::new(link)
    }).collect();

    let results = future::join_all(email_futures).await;

    results.into_iter().filter_map(Result::ok).collect()
}

async fn notion_links() -> Result<Vec<notion::NotionLink>, Box<dyn std::error::Error>>{
    let notion_links_file = Path::new(NOTION_LINKS_FILENAME);

    if notion_links_file.exists() {
        println!("links already saved! no need to fetch");
        let notion_links = fs::read_to_string(NOTION_LINKS_FILENAME).unwrap();
        let notion_links: Vec<notion::NotionLink> = serde_json::from_str(&notion_links).unwrap();
        return Ok(notion_links);
    } else {
        println!("getting links from notion");
        let notion_links = get_notion_links().await?;
        save_notion_links_to_file(&notion_links);
        return Ok(notion_links);
    }
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
