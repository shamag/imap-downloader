extern crate imap;
extern crate native_tls;
extern crate mailparse;
extern crate structopt;
extern crate rpassword;
extern crate pbr;

// use std::fs::File;
use std::io::Write;
use std::fs::OpenOptions;
use std::path::Path;
use structopt::StructOpt;
// use std::io::BufRead;
use rpassword::read_password;
use pbr::ProgressBar;

#[derive(Debug, StructOpt)]
#[structopt(name = "imap-downloader", about = "simple cli-app for downloading emails")]
struct Opt {
    /// server domain (imap.yandex.ru)
    #[structopt(short = "d", long = "domian")]
    domain: String,
    /// login (some@mail.ex)
    #[structopt(short = "l", long = "login")]
    login: String,
    /// server directory to download (INBOX)
    #[structopt(short = "m", long = "mailbox", default_value="INBOX")]
    mailbox: String,
    /// directory to save messages
    #[structopt(short = "o", long = "output", default_value="out")]
    output: String,
}

struct FetchOpts {
    domain: String,
    login: String,
    mailbox: String,
    output: String,
    password: String
}

fn main() -> imap::error::Result<(), > {
    let opt = Opt::from_args();
    println!("Type a password: ");
    let password = read_password().unwrap();
    let imap_opts = FetchOpts{
        domain: opt.domain,
        login: opt.login,
        mailbox: opt.mailbox,
        output: opt.output,
        password: password,
    };
    fetch_dir(imap_opts)
}

fn fetch_dir(opts: FetchOpts) -> imap::error::Result<(), > {
    // let domain = "imap.yandex.ru";
    let domain = &opts.domain;
    let out_path = Path::new(&opts.output);
    std::fs::create_dir_all(out_path)?;
    let ext = "eml";
    let tls = native_tls::TlsConnector::builder().build()?;

    let client = imap::connect((&domain[..], 993), &domain[..], &tls).unwrap();

    let mut imap_session = client
        .login(opts.login, opts.password)
        .map_err(|e| e.0)?;

    imap_session.examine(opts.mailbox)?;
    let count = imap_session.search("1:*")?.len();
    println!("{} messages was found, fetching...", &count);
    let mut pb = ProgressBar::new((count + 1) as u64);
    pb.inc();
    pb.format("╢▌▌░╟");
    let messages = imap_session.fetch("1:*", "RFC822")?;
    let _res: Vec<std::io::Result<(), >> = messages.iter()
        .map(|m| {
            // dbg!(m.message);
            let body = if let Some(body) = m.body() {
                body
            } else {
                return Ok(())
            };
            let file_name = &format!("{}.{}", m.message, ext);
            let path = out_path.join(file_name);
            pb.inc();
            let mut file = OpenOptions::new().write(true).create(true).open(path)?;
            file.write_all(body)
            })
        .collect();
    // dbg!(res);
    imap_session.logout()?;
    pb.finish_print("done");
    Ok(())
}