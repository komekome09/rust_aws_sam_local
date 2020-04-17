#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate reqwest;
extern crate scraper;

use lambda::error::HandlerError;
use std::error::Error;
use simple_error::bail;
use scraper::{Selector, Html};
use std::io::Read;

#[derive(Deserialize, Clone)]
struct CustomEvent {
    #[serde(rename = "firstName")]
    first_name: String,
}

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    lambda!(my_handler);

    Ok(())
}

#[derive(Debug)]
struct ProductInfo {
    name: String,
    price: String,
    rest: String,
}

fn my_handler(e: CustomEvent, c: lambda::Context) -> Result<CustomOutput, HandlerError> {
    let client = reqwest::blocking::Client::new();
    let url = "https://www.amazon.co.jp/dp/B079211FWH";
    let body_json = client.get(url) 
                    .send()
                    .unwrap()
                    .text()
                    .unwrap();
    let parse_doc = Html::parse_document(&body_json);

    let mut product_name = String::new();
    for node in parse_doc.select(&Selector::parse("span#productTitle").unwrap()) {
        product_name = node.inner_html().clone();
    }
    
    let mut price = String::new();
    for node in parse_doc.select(&Selector::parse("span#priceblock_ourprice").unwrap()) {
        price = node.inner_html().clone();
    }

    let mut rest = String::new();
    for node in parse_doc.select(&Selector::parse("#availability > span:nth-child(1)").unwrap()) {
        rest = node.inner_html().clone();
    }

    let info = ProductInfo {
        name: product_name.trim().to_string(),
        price: price.trim().to_string(),
        rest: rest.trim().to_string(),
    };

    let post_client = reqwest::blocking::Client::new();
    let body = format!(r#"{{"text": "name: {},price: {},rest: {}"}}"#, info.name, info.price, info.rest);
    let mut post_slack = client.post("https://hooks.slack.com/services/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")
                            .header(reqwest::header::CONTENT_TYPE, "application/json")
                            .body(body)
                            .send()
                            .unwrap();
    let mut buf = String::new();
    post_slack.read_to_string(&mut buf).expect("Failed to read response");

    Ok(CustomOutput {
        message: format!("{:?}\n{}", info, buf),
    })
}
