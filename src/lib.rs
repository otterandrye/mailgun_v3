extern crate chrono;
extern crate reqwest;
#[macro_use] extern crate serde_derive;

pub mod email;

use reqwest::Error as ReqError;

pub type MailgunResult<T> = Result<T, ReqError>;

pub struct Credentials {
    api_key: String,
    domain: String,
}

impl Credentials {
    pub fn new(api_key: &str, domain: &str) -> Self {
        Credentials { api_key: api_key.to_string(), domain: domain.to_string() }
    }
}
