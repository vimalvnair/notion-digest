use lettre::{Transport, SmtpTransport, Message, message::header::ContentType};

pub fn send_local_mail(email_body: String) {
    let smtp_server = "127.0.0.1";
    let smtp_port = 1025;

    let mailer = SmtpTransport::builder_dangerous(smtp_server).port(smtp_port).build();

    let email = Message::builder()
        .from("notion@example.com".parse().unwrap())
        .to("recipient@example.com".parse().unwrap())
        .subject("Notion digest")
        .header(ContentType::TEXT_HTML)
        .body(email_body)
        .unwrap();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent!"),
        Err(e) => eprintln!("Failed to send email: {:?}", e),
    }
}
