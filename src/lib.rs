//! `reqwest` based web bindings for Mailgun's [v3 JSON API](https://documentation.mailgun.com/en/latest/api_reference.html)
//!
//! This crate wraps some of Mailgun's APIs, but doesn't attempt to do much else
//! in terms of error handling or argument sanitization

extern crate chrono;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;

pub mod email;
pub mod validation;
pub mod templates;

pub use reqwest::Error as ReqError;

const MAILGUN_DEFAULT_API: &str = "https://api.mailgun.net/v3";

///! Wrapper result type returning `reqwest` errors
pub type MailgunResult<T> = Result<T, ReqError>;

///! Mailgun private API key and sending domain
#[derive(Debug)]
pub struct Credentials {
    api_base: String,
    api_key: String,
    domain: String,
}

impl Credentials {
    pub fn new<A: AsRef<str>, D: AsRef<str>>(api_key: A, domain: D) -> Self {
        Self::with_base(MAILGUN_DEFAULT_API, api_key, domain)
    }
    pub fn with_base<B: AsRef<str>, A: AsRef<str>, D: AsRef<str>>(
        api_base: B,
        api_key: A,
        domain: D,
    ) -> Self {
        let api_base = api_base.as_ref();
        let api_key = api_key.as_ref();
        let domain = domain.as_ref();
        assert!(
            api_base.starts_with("http"),
            "Domain does not start with http"
        );
        assert!(
            api_base.chars().filter(|c| *c == '.').count() >= 1,
            "api_base does not contain any dots"
        );
        assert!(api_key.len() >= 35, "api_key is to short");
        assert!(
            domain.chars().filter(|c| *c == '.').count() >= 1,
            "Domain does not contain any dots"
        );
        Credentials {
            api_base: api_base.to_string(),
            api_key: api_key.to_string(),
            domain: domain.to_string(),
        }
    }
    pub fn domain(&self) -> &str {
        &self.domain
    }
}

///! An email address, with or without a display name
#[derive(Debug)]
pub struct EmailAddress {
    name: Option<String>,
    address: String,
}

impl EmailAddress {
    pub fn address<T: ToString>(address: T) -> Self {
        EmailAddress {
            name: None,
            address: address.to_string(),
        }
    }

    pub fn name_address<T: ToString>(name: T, address: T) -> Self {
        EmailAddress {
            name: Some(name.to_string()),
            address: address.to_string(),
        }
    }

    pub fn email(&self) -> &str {
        &self.address
    }

    pub fn to_string(&self) -> String {
        match self.name {
            Some(ref name) => format!("{} <{}>", name, self.address),
            None => self.address.clone(),
        }
    }
}
