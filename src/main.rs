use std::fs;
use std::path::Path;
use clap::{Parser, crate_name};
use serde::Deserialize;

use lettre::Message;
use lettre::message::Attachment;
use lettre::message::header::{ContentType, To, Cc, Bcc};
use lettre::message::{Mailbox, Mailboxes};
use lettre::message::{SinglePart, MultiPart};

use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};

use platform_dirs::AppDirs;
use mime;

#[derive(Deserialize)]
struct Config {
    name: String,
    email: String,
    smtp: ServerConfig,

    #[allow(unused)]
    imap: ServerConfig
}

#[derive(Deserialize)]
struct ServerConfig {
    hostname: String,
    username: String,

    #[allow(unused)]
    port: u16,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The account to use, defined in `~/config/mail/`.
    #[arg()]
    account: String,

    /// Path to the body contents of the email, markdown is assumed and sent as HTML.
    #[arg()]
    path: String,

    /// Password for the SMTP account.
    #[arg(short, long)]
    password: String,

    /// `Subject` header.
    #[arg(short, long)]
    subject: String,

    /// `To` header: main recipient(s) for the email.
    #[arg(long, required=true)]
    to: Vec<String>,

    /// `CC` (Carbon copy) header: send a copy of the email to these email addresses.
    #[arg(long)]
    cc: Vec<String>,

    /// `BCC` (Blind carbon copy) header: same as CC, but the main recipient(s) can't see it.
    #[arg(long)]
    bcc: Vec<String>,

    /// Attach a file to the email.
    #[arg(short, long)]
    attach: Vec<String>
}

fn get_config(account: String) -> Config {
    let directories = AppDirs::new(Some(crate_name!()), false).unwrap();

    let config_file = directories.config_dir.join(account);
    let toml = fs::read_to_string(config_file).expect("Couldn't read config file.");

    toml::from_str(&toml).expect("Failed to parse TOML.")        
}

fn main() {
    let args = Args::parse();
    let config = get_config(args.account);

    let mail = create_mail(
        args.path, 
        args.subject, 
        args.to, 
        args.cc, 
        args.bcc, 
        args.attach, 
        &config
    );

    send_mail(mail, args.password, &config)
}

fn create_mail(path: String, subject: String, to: Vec<String>, cc: Vec<String>, bcc: Vec<String>, files: Vec<String>, config: &Config) -> Message {
    let from = parse_address(format!("{} <{}>", config.name, config.email));
    
    let to: To = addresses(to).into();
    let cc: Cc = addresses(cc).into();
    let bcc: Bcc = addresses(bcc).into();

    let (plain, html) = parse_markdown(path);

    let body = MultiPart::alternative_plain_html(plain, html);
    let mut content = MultiPart::mixed().multipart(body);
    
    for file in files {
        let attachment = create_attachment(file);
        content = content.singlepart(attachment);
    };

    Message::builder()
        .from(from)
        .subject(subject)
        .mailbox(to)
        .mailbox(cc)
        .mailbox(bcc)
        .multipart(content)
        .expect("Failed to build message.")
}

fn addresses(addresses: Vec<String>) -> Mailboxes {
    let mut mailboxes = Mailboxes::new();
    for address in addresses {
        let mailbox = parse_address(address);
        mailboxes.push(mailbox);
    }

    mailboxes
}

fn create_attachment(path: String) -> SinglePart {
    validate_file(&path);

    let basename = Path::new(&path).file_name().unwrap().to_str().unwrap().to_string();
    let body = fs::read(&path).expect(&format!("{}: Couldn't read file.", path));

    // Try to infer the mime type and otherwise fall back to application/octet-stream
    let mime_type = mime_guess::from_path(&path).first().unwrap_or(mime::APPLICATION_OCTET_STREAM);
    let content_type = ContentType::parse(&mime_type.to_string()).unwrap();
    
    Attachment::new(basename).body(body, content_type)
}

fn parse_address(address: String) -> Mailbox {
    address.parse().expect(&format!("Malformed address: {}", address))
}

fn parse_markdown(path: String) -> (String, String) {
    validate_file(&path);

    let plain = fs::read_to_string(&path).expect(&format!("{}: Couldn't read file.", path));
    let html = markdown::to_html(&plain);

    (plain, html)
}

fn validate_file(path: &str) {
    let file = Path::new(path);
    if !file.exists() { panic!("{}: No such file or directory.", path) }
    if !file.is_file() { panic!("{}: Not a file.", path) }
}

fn send_mail(mail: Message, password: String, config: &Config) {
    let credentials = Credentials::new(config.smtp.username.clone(), password);
    let mailer = SmtpTransport::relay(&config.smtp.hostname)
        .unwrap()
        .credentials(credentials)
        .build();

    match mailer.send(&mail) {
        Ok(_) => println!("Sent!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}
