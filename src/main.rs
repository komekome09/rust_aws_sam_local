use lambda_runtime::{handler_fn, Context, Error};
use serde_json::{json, Value};
use scraper::{Selector, Html};
use reqwest::header::{HeaderMap, CONTENT_TYPE, AUTHORIZATION};
use std::env;

#[derive(Debug)]
struct ProductInfo {
    name: String,
    price: String,
    rest: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    simple_logger::init_with_level(log::Level::Info)?;
    let func = handler_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: Value, _: Context) -> Result<Value, Error> {
    let url = match env::var("PRODUCT_URL") {
        Ok(val) => val,
        Err(err) => panic!("{}: {}", err, "PRODUCT_URL")
    };
    let first_name = event["firstName"].as_str().unwrap_or("world");
    let res_body = reqwest::get(url).await?.text().await?;
    let parse_doc = Html::parse_document(&res_body);

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

    let token = match env::var("SLACK_API_TOKEN") {
        Ok(val) => val,
        Err(err) => {
            panic!("{}: {}", err, "SLACK_API_TOKEN");
        }
    };

    
    let client = reqwest::Client::new();
    let mut slack_headermap = HeaderMap::new();
    slack_headermap.insert(CONTENT_TYPE, "application/x-www-urlencoded".parse().unwrap());
    slack_headermap.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    slack_headermap.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());

    let authtest_res = client.post("https://slack.com/api/auth.test")
        .headers(slack_headermap)
        .send()
        .await;

    match authtest_res {
        Ok(_) => println!("Auth no problem"),
        Err(_) => panic!("Auth failed. Please check api token"),
    };

    drop(authtest_res);

    let channel = match env::var("SLACK_CHANNEL_ID") {
        Ok(val) => val,
        Err(err) => panic!("{}: {}", err, "SLACK_CHANNEL_ID"),
    };
    let post_msg = [("channel", &channel), ("text", &format!("Product Name: {}\nNow Price: {}\nRest {}", info.name, info.price, info.rest))];

    let mut slack2_headermap = HeaderMap::new();
    slack2_headermap.insert(CONTENT_TYPE, "application/x-www-urlencoded".parse().unwrap());
    slack2_headermap.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    slack2_headermap.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
    let post_res = client.post("https://slack.com/api/chat.postMessage")
        .headers(slack2_headermap)
        .form(&post_msg)
        .send()
        .await?;

    let res_json: Value = post_res.json().await?;
    Ok(json!({
        "message": format!("Hello, {}!", first_name),
        "response": format!("{:?}", res_json),
    }))
}
