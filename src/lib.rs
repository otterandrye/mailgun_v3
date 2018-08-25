//! `reqwest` based web bindings for Mailgun's [v3 JSON API](https://documentation.mailgun.com/en/latest/api_reference.html)
//!
//! This crate wraps some of Mailgun's APIs, but doesn't attempt to do much else
//! in terms of error handling or argument sanitization

extern crate chrono;
extern crate reqwest;
#[macro_use] extern crate serde_derive;

pub mod email;
pub mod validation;

pub use reqwest::Error as ReqError;

const MAILGUN_API: &str = "https://api.mailgun.net/v3";

///! Wrapper result type returning `reqwest` errors
pub type MailgunResult<T> = Result<T, ReqError>;

///! Mailgun private API key and sending domain
pub struct Credentials {
    api_key: String,
    domain: String,
}

impl Credentials {
    pub fn new(api_key: &str, domain: &str) -> Self {
        Credentials { api_key: api_key.to_string(), domain: domain.to_string() }
    }
}

///! An email address, with or without a display name
pub struct EmailAddress {
    name: Option<String>,
    address: String,
}

impl EmailAddress {
    pub fn address(address: &str) -> Self {
        EmailAddress { name: None, address: address.to_string() }
    }

    pub fn name_address(name: &str, address: &str ) -> Self {
        EmailAddress { name: Some(name.to_string()), address: address.to_string() }
    }

    pub fn email(&self) -> &str {
        &self.address
    }

    pub fn to_string(&self) -> String {
        match self.name {
            Some(ref name) => format!("{} <{}>", name, self.address),
            None => self.address.clone()
        }
    }
}
