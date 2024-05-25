use std::collections::HashMap;
use std::str::FromStr;
use clap::Parser;
use anyhow::{Result, anyhow};
use colored::Colorize;
use mime::Mime;
use reqwest::{Client, header, Response, Url};

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "author")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// A help message for the `get` subcommand
    Get(Get),
    /// A help message for the `post` subcommand
    Post(Post),
}

#[derive(Parser, Debug)]
struct Get {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

#[derive(Parser, Debug)]
struct Post {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    #[clap(parse(try_from_str = parse_kvs))]
    body: Vec<KvItem>,
}

fn parse_url(url: &str) -> Result<String> {
    let _url: Url = url.parse()?;
    Ok(url.into())
}

#[derive(Debug)]
struct KvItem {
    key: String,
    value: String,
}

impl FromStr for KvItem {
    type Err = anyhow::Error;
    // 实现 `from_str` 方法，用于将字符串解析为 `KvItem`
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            key: (split.next().ok_or_else(err)?).to_string(),
            value: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_kvs(s: &str) -> Result<KvItem> {
    Ok(s.parse()?)
}

async fn get(client: Client, args: &Get) -> Result<Response> {
    let resp = client.get(&args.url).send().await?;
    Ok(resp)
}

async fn post(client: Client, args: &Post) -> Result<Response> {
    let mut body = HashMap::new();
    for item in &args.body {
        body.insert(&item.key, &item.value);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(resp)
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    println!();
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan());
        }
        _ => {
            println!("{}", body)
        }
    }
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers().get(header::CONTENT_TYPE).map(|ct| ct.to_str().unwrap().parse().unwrap())
}

async fn print_response(resp: reqwest::Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let mut headers = header::HeaderMap::new();
    // 为我们的 HTTP 客户端添加一些缺省的 HTTP 头
    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);
    let client = reqwest::Client::builder().default_headers(headers).build()?;
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };
    Ok(print_response(result).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        assert!(parse_url("https://www.baidu.com").is_ok());
        assert!(parse_url("www.baidu.com").is_err());
    }

    #[test]
    fn test_parse_kvs() {
        assert!(parse_kvs("key=value").is_ok());
        assert!(parse_kvs("key").is_err());
    }
}