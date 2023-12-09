# Notion Links Digest

This Rust application is designed to help you revisit the links you've saved in Notion. It randomly selects a configurable number of links(default: 3) from your "My links" database and sends them via email, allowing you to discover or re-discover valuable resources you've saved over time.

## Environment Variables

Configure the application using these environment variables:

- `NOTION_API_KEY`: Your Notion API key, obtained from [Notion Developers](https://developers.notion.com/).
- `NOTION_DATABASE_ID`: The ID of your "My links" database in Notion.
- `SENDGRID_API_KEY`: Your SendGrid API key for email sending capabilities.
- `FROM_ADDRESS`: The sender's email address for the digest.
- `TO_ADDRESS`: The intended recipient's email address for the digest.
- `NUMBER_OF_LINKS_TO_FETCH`: The number of random links to fetch from your "My links" database for the digest email. Default is 3 if not set.

## Setup Instructions

### Notion API Key and Database Connection

1. Go to [Notion Developers](https://developers.notion.com/) and create an integration.
2. Use the provided API key as your `NOTION_API_KEY`.
3. Share your "My links" database with the integration you created.
  <img width="244" alt="" src="https://github.com/vimalvnair/notion-digest/assets/1711390/88e8f7fc-6bb2-475a-acb2-0d52c373d659">


## Example email preview
<img width="665" alt="" src="https://github.com/vimalvnair/notion-digest/assets/1711390/4a6edcf7-1106-4e01-9ae1-9d2150b22954">


### Running the Application

To run this application automatically, you can use a scheduler like cron. Here's an example of a cron job that runs the application every day at a specified time:

```crontab
0 8 * * * /path/to/your/app
```

### License

This project is released under the [MIT License](https://opensource.org/license/mit/).
