use std::env;
use serde_json::json;

pub async fn send_email(email_body: String) -> Result<(), Box<dyn std::error::Error>>{
  let sendgrid_api_key = env::var("SENDGRID_API_KEY").unwrap();
  let from_address = env::var("FROM_ADDRESS").unwrap();
  let to_address = env::var("TO_ADDRESS").unwrap();

  let url = "https://api.sendgrid.com/v3/mail/send";

  let request_params = json!({
    "personalizations": [
      {
        "to": [
          {
            "email": to_address
          }
        ]
      }
    ],
    "from": {
      "email": from_address
    },
    "subject": "Notion Digest",
    "content": [
      {
        "type": "text/html",
        "value": email_body
      }
    ]
  });

  let response = reqwest::Client::new()
  .post(url)
  .header("Content-Type", "application/json")
  .json(&request_params)
  .bearer_auth(&sendgrid_api_key)
  .send()
  .await?;

  println!("sendgrid response = {:?}", response.status());

  Ok(())
}
