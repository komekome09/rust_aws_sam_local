#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate rewqest;

use lambda::error::HandlerError;
use std::error::Error;
use simple_error::bail;
use std::net;
use std::io::{Write,Read};

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

fn my_handler(e: CustomEvent, c: lambda::Context) -> Result<CustomOutput, HandlerError> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        bail!("Empty first name");
    }

    let body = reqwest::get("https://www.dropbox.com/oauth2/authorize")
                        .await?
                        .text()
                        .await?;

    Ok(CustomOutput {
        message: format!("Hello, {}!\n{:?}", e.first_name, body),
    })
}
