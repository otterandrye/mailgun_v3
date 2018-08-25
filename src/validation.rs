//! Validate emails through Mailgun, to reduce bounce rate, find typos, etc

use reqwest;
use std::collections::HashMap;

use ::{Credentials, MailgunResult, MAILGUN_API};

///! Returned for sucessfully parsed email addresses
#[derive(Deserialize, Debug)]
pub struct EmailParts {
    domain: String,
    display_name: Option<String>,
    local_part: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidationResponse {
    pub address: String,
    pub did_you_mean: Option<String>,
    pub is_disposable_address: bool,
    pub is_role_address: bool,
    pub is_valid: bool,
    pub parts: Option<EmailParts>,
    pub reason: Option<String>,
}

const VALIDATION_ENDPOINT: &str = "address/private/validate";

// curl -G --user 'api:pubkey-5ogiflzbnjrljiky49qxsiozqef5jxp7' -G \
//     https://api.mailgun.net/v3/address/validate \
//     --data-urlencode address='foo@mailgun.net'
/// Validate an email using mailgun's validation service
/// [API docs](https://documentation.mailgun.com/en/latest/api-email-validation.html#email-validation)
pub fn validate_email(creds: &Credentials, address: &str) -> MailgunResult<ValidationResponse> {
    let client = reqwest::Client::new();
    validate_email_with_client(&client, creds, address)
}

/// Same as `validate_email` but with an externally managed client
pub fn validate_email_with_client(client: &reqwest::Client, creds: &Credentials, address: &str) -> MailgunResult<ValidationResponse> {
    let url = format!("{}/{}", MAILGUN_API, VALIDATION_ENDPOINT);
    let mut params = HashMap::new();
    params.insert("address".to_string(), address);

    let mut res = client.get(&url)
        .basic_auth("api", Some(creds.api_key.clone()))
        .form(&params)
        .send()?
        .error_for_status()?;

    let parsed: ValidationResponse = res.json()?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn run_validate_email() {
        // add your api key here to run the tests - accounts get 100 validations/month free
        let key = "something-secret-something-safe";
        let creds = Credentials::new(&key, "not needed");

        let res = validate_email(&creds, "james.earl.jones@gmail.com");
        assert!(res.is_ok(), format!("{:?}", &res));
        let parsed = res.unwrap();
        print!("got response: {:?}", parsed);
        assert_eq!(parsed.address, "james.earl.jones@gmail.com");
        assert!(parsed.is_valid);
        assert!(!parsed.is_disposable_address);
        assert!(!parsed.is_role_address);
        assert_eq!(parsed.reason, None);
    }
}
