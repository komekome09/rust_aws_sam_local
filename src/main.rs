#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate reqwest;

use lambda::error::HandlerError;
use std::error::Error;
use simple_error::bail;
use std::env;

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

#[derive(Serialize, Deserialize)]
struct AuthToken {
    token_type: String,
    expires_in: u32,
    ext_expires_in: u32,
    access_token: String,
}

fn my_handler(e: CustomEvent, c: lambda::Context) -> Result<CustomOutput, HandlerError> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        bail!("Empty first name");
    }

    let client_id = match env::var("CLIENT_ID") {
        Ok(n) => n,
        Err(_) => bail!("CLIENT_ID not found")
    };
    let secret_key = match env::var("CLIENT_SECRET") {
        Ok(n) => n,
        Err(_) => bail!("CLIENT_SECRET not found")
    };
    let tenant_id = match env::var("TENANT_ID") {
        Ok(n) => n,
        Err(_) => bail!("TENANT_ID not found")
    };

    let params = [("client_id", client_id.as_str()),
                    ("scope", "https://graph.microsoft.com/.default"),
                    ("client_secret", secret_key.as_str()),
                    ("grant_type", "client_credentials")];
    let client = reqwest::blocking::Client::new();
    let url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id);
    let body_json = client.post(&url)
                    .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .form(&params)
                    .send()
                    .unwrap()
                    .text()
                    .unwrap();

    let json: AuthToken = match serde_json::from_str(&body_json) {
        Ok(n) => n,
        Err(n) => {
            error!("Error: {:?}", n);
            bail!("Fail get token");
        }
    };

    Ok(CustomOutput {
        message: format!("{}", json.access_token),
    })
}
