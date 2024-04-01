use std::fs;
use clap::{Parser, crate_name};
use serde::Deserialize;

use lettre::Message;
use lettre::message::header;
use lettre::message::{Mailbox, Mailboxes};
use lettre::message::{SinglePart, MultiPart};

use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};

use platform_dirs::AppDirs;

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
}

fn get_config(account: String) -> Config {
    let directories = AppDirs::new(Some(crate_name!()), false).unwrap();
    let config_file = format!("{}.toml", account);
    let config_path = directories.config_dir.join(config_file);
    
    let toml = fs::read_to_string(config_path).expect("Couldn't read config file.");

    toml::from_str(&toml).expect("Failed to parse TOML.")        
}

fn main() {
    let args = Args::parse();
    let config = get_config(args.account);

    let mail = create_mail(args.path, args.subject, args.to, args.cc, args.bcc, &config);

    send_mail(mail, args.password, &config)
}

fn create_mail(path: String, subject: String, to: Vec<String>, cc: Vec<String>, bcc: Vec<String>, config: &Config) -> Message {
    let from = parse_address(format!("{} <{}>", config.name, config.email));
    
    let to: header::To = addresses(to).into();
    let cc: header::Cc = addresses(cc).into();
    let bcc: header::Bcc = addresses(bcc).into();

    let (plain, html) = parse_markdown(path);

    let plain_part = SinglePart::plain(plain);
    let html_part = SinglePart::html(html);

    let body = MultiPart::alternative().singlepart(plain_part).singlepart(html_part);

    Message::builder()
        .from(from)
        .subject(subject)
        .mailbox(to)
        .mailbox(cc)
        .mailbox(bcc)
        .multipart(body)
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

fn parse_address(address: String) -> Mailbox {
    address.parse().expect(&format!("Malformed address: {}", address))
}

fn parse_markdown(path: String) -> (String, String) {
    let plain = fs::read_to_string(&path).expect(&format!("{}: No such file or directory.", path));
    let html = markdown::to_html(&plain);

    (plain, html)
}

fn send_mail(mail: Message, password: String, config: &Config) {
    let credentials = Credentials::new(config.smtp.username.clone(), password);
    let mailer = SmtpTransport::relay(&config.smtp.hostname)
        .unwrap()
        .credentials(credentials)
        .build();

    match mailer.send(&mail) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}
