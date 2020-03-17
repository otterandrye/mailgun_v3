//! Send email through Mailgun

use chrono::prelude::*;
use reqwest;
use std::collections::HashMap;

use crate::{Credentials, MailgunResult, MAILGUN_API};
pub use crate::EmailAddress;

///! `Html` and `Text` emails use different API params
pub enum MessageBody {
    Html(String),
    Text(String),
    HtmlAndText(String, String),
}

impl Default for MessageBody {
    fn default() -> MessageBody { MessageBody::Text(String::from("")) }
}

impl MessageBody {
    fn add_to(self, params: &mut HashMap<String, String>) {
        match self {
            MessageBody::Html(c) => params.insert(String::from("html"), c),
            MessageBody::Text(c) => params.insert(String::from("text"), c),
            MessageBody::HtmlAndText(html, text) => {
                params.insert(String::from("html"), html);
                params.insert(String::from("text"), text)
            },
        };
    }
}

///! An email to send through Mailgun. Won't send without a body
#[derive(Default)]
pub struct Message {
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    pub body: MessageBody,
    pub options: Vec<SendOptions>,
}

impl Message {
    fn to_params(self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        Message::add_recipients("to", self.to, &mut params);
        Message::add_recipients("cc", self.cc, &mut params);
        Message::add_recipients("bcc", self.bcc, &mut params);

        params.insert(String::from("subject"), self.subject);

        self.body.add_to(&mut params);

        for opt in self.options {
            opt.add_to(&mut params);
        }

        params
    }

    fn add_recipients(field: &str, addresses: Vec<EmailAddress>, params: &mut HashMap<String, String>) {
        if !addresses.is_empty() {
            let joined = addresses.iter()
                .map(EmailAddress::to_string)
                .collect::<Vec<String>>()
                .join(",");
            params.insert(field.to_owned(), joined);
        }
    }
}

///! Some of the parameters exposed by the mailgun send API
pub enum SendOptions {
    TestMode, // o:testmode
    DeliveryTime(DateTime<Utc>), // o:deliverytime
    Header(String, String), // h:X-My-Header
    Tag(String), // o:tag
}

impl SendOptions {
    fn add_to(&self, params: &mut HashMap<String, String>) {
        use self::SendOptions::*;
        let (key, value) = match self {
            TestMode => (String::from("o:testmode"), String::from("yes")),
            DeliveryTime(instant) => (String::from("o:deliverytime"), instant.to_rfc2822()),
            Header(header, val) => {
                let key = format!("h:{}", header);
                (key, val.to_owned())
            },
            Tag(tag) => (String::from("o:tag"), tag.to_owned()),
        };
        params.insert(key, value);
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SendResponse {
    pub message: String,
    pub id: String,
}

const MESSAGES_ENDPOINT: &str = "messages";

// curl -s --user 'api:YOUR_API_KEY' \
//     https://api.mailgun.net/v3/YOUR_DOMAIN_NAME/messages \
//     -F from='Excited User <mailgun@YOUR_DOMAIN_NAME>' \
//     -F to=YOU@YOUR_DOMAIN_NAME \
//     -F to=bar@example.com \
//     -F subject='Hello' \
//     -F text='Testing some Mailgun awesomeness!'
/// Sends a single email from the specified sender address
/// [API docs](https://documentation.mailgun.com/en/latest/api-sending.html#sending)
pub fn send_email(creds: &Credentials, sender: &EmailAddress, msg: Message) ->  MailgunResult<SendResponse> {
    let client = reqwest::blocking::Client::new();
    send_with_client(&client, creds, sender, msg)
}

/// Same as `send_email` but with an externally managed client
pub fn send_with_client(client: &reqwest::blocking::Client, creds: &Credentials, sender: &EmailAddress, msg: Message) -> MailgunResult<SendResponse> {
    let url = format!("{}/{}/{}", MAILGUN_API, creds.domain, MESSAGES_ENDPOINT);
    let request_builder = client.post(&url);
    send_with_request_builder(request_builder, creds, sender, msg)
}

/// Same as `send_email` but with an externally managed request builder.
/// Use this in case you want to send the mails to a custom API endpoint, e.g. for testing.
pub fn send_with_request_builder(request_builder: reqwest::blocking::RequestBuilder, creds: &Credentials, sender: &EmailAddress, msg: Message) -> MailgunResult<SendResponse> {
    let mut params = msg.to_params();
    params.insert("from".to_string(), sender.to_string());

    let res = request_builder
        .basic_auth("api", Some(creds.api_key.clone()))
        .form(&params)
        .send()?
        .error_for_status()?;

    let parsed: SendResponse = res.json()?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use super::*;
    use serde_json::json;

    #[test]
    fn message_body() {
        let text = Message {
            body: MessageBody::Text(String::from("hello, world")),
            ..Default::default()
        };
        let params = text.to_params();
        assert_eq!(params.get("text"), Some(&String::from("hello, world")));

        let html = Message {
            body: MessageBody::Html(String::from("<body>hello, world</body>")),
            ..Default::default()
        };
        let params = html.to_params();
        assert_eq!(params.get("html"), Some(&String::from("<body>hello, world</body>")));

        let both = Message {
            body: MessageBody::HtmlAndText(String::from("<body/>"), String::from("hello")),
            ..Default::default()
        };
        let params = both.to_params();
        assert_eq!(params.get("html"), Some(&String::from("<body/>")));
        assert_eq!(params.get("text"), Some(&String::from("hello")));
    }

    #[test]
    fn message_recipients() {
        let msg = Message {
            to: vec![EmailAddress::address("foo@bar.com")],
            cc: vec![EmailAddress::name_address("Tim", "woo@woah.com"), EmailAddress::address("z@c.c")],
            ..Default::default()
        };

        let params = msg.to_params();
        assert_eq!(params.get("to"), Some(&String::from("foo@bar.com")));
        assert_eq!(params.get("cc"), Some(&String::from("Tim <woo@woah.com>,z@c.c")));
        assert_eq!(params.get("bcc"), None);
    }

    #[test]
    fn send_options() {
        let msg = Message {
            options: vec![
              SendOptions::TestMode,
              SendOptions::DeliveryTime(Utc.timestamp_millis(1431648000)),
              SendOptions::Header("X-For".to_owned(), "Fizz".to_owned()),
              SendOptions::Tag("Important".to_owned()),
            ],
            ..Default::default()
        };

        let params = msg.to_params();
        assert_eq!(params.get("o:testmode"), Some(&String::from("yes")));
        assert_eq!(params.get("o:deliverytime"), Some(&String::from("Sat, 17 Jan 1970 13:40:48 +0000")));
        assert_eq!(params.get("h:X-For"), Some(&String::from("Fizz")));
        assert_eq!(params.get("o:tag"), Some(&String::from("Important")));
    }

    #[test]
    fn request_unauthorized() {
        // invalid key & domain
        let creds = Credentials::new("key-your_key_here", "aksdfa32undkjns.com");
        let recipient = EmailAddress::address("timmy@aksdfa32undkjns.com");
        let message = Message {
            to: vec![recipient],
            subject: "Test email".to_string(),
            ..Default::default()
        };
        let sender = EmailAddress::name_address("Nick Testla", "nick@aksdfa32undkjns.com");

        let res = send_email(&creds, &sender, message);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().status(), Some(StatusCode::UNAUTHORIZED));
    }

    #[ignore]
    #[test]
    fn actually_send_email() {
        // if you want to try actually sending an email w/ your credentials add them to this test
        // and run it.
        let domain = "sandbox-some_numbers_here_probably.mailgun.org";
        let key = "something-secret-something-safe";
        let recipient = "foo@bar.com";

        let creds = Credentials::new(&key, &domain);
        let recipient = EmailAddress::address(&recipient);
        let message = Message {
            to: vec![recipient],
            subject: "Test email".to_string(),
            body: MessageBody::Text(String::from("This email is from an mailgun_v3 automated test")),
            ..Default::default()
        };
        let sender = EmailAddress::name_address("Nick Testla", &format!("mailgun_v3@{}", &domain));

        let res = send_email(&creds, &sender, message);
        assert!(res.is_ok(), format!("{:?}", &res));
    }

    #[test]
    fn test_send_with_request_builder() {
        let domain = "sandbox0123456789abcdef0123456789abcdef.mailgun.org";
        let key = "0123456789abcdef0123456789abcdef-01234567-89abcdef";
        let recipient = "user@example.com";

        let creds = Credentials::new(&key, &domain);
        let recipient = EmailAddress::address(&recipient);
        let message = Message {
            to: vec![recipient],
            subject: "Test email".to_string(),
            body: MessageBody::Text(String::from("This email is from an mailgun_v3 automated test")),
            ..Default::default()
        };
        let sender = EmailAddress::name_address("Nick Testla", &format!("mailgun_v3@{}", &domain));

        let domain = &mockito::server_url();
        let uri = format!("/{}/{}", creds.domain, MESSAGES_ENDPOINT);

        let response = json!({
            "id": "<0123456789abcdef.0123456789abcdef@sandbox0123456789abcdef0123456789abcdef.mailgun.org>",
            "message": "Queued. Thank you."
        });
        let _m = mockito::mock("POST", uri.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create();

        let url = format!("{}{}", domain, uri);
        let client = reqwest::blocking::Client::new();
        let request_builder = client.post(&url);
        let res = send_with_request_builder(request_builder, &creds, &sender, message);
        assert!(res.is_ok(), format!("{:?}", &res));
    }
}

pub mod async_impl {
    use super::*;

    /// Sends a single email from the specified sender address
    /// [API docs](https://documentation.mailgun.com/en/latest/api-sending.html#sending)
    pub async fn send_email(
        creds: &Credentials,
        sender: &EmailAddress,
        msg: Message,
    ) -> MailgunResult<SendResponse> {
        let client = reqwest::Client::new();
        send_with_client(&client, creds, sender, msg).await
    }

    /// Same as `send_email` but with an externally managed client
    pub async fn send_with_client(
        client: &reqwest::Client,
        creds: &Credentials,
        sender: &EmailAddress,
        msg: Message,
    ) -> MailgunResult<SendResponse> {
        let url = format!("{}/{}/{}", MAILGUN_API, creds.domain, MESSAGES_ENDPOINT);
        let request_builder = client.post(&url);
        send_with_request_builder(request_builder, creds, sender, msg).await
    }

    /// Same as `send_email` but with an externally managed request builder.
    /// Use this in case you want to send the mails to a custom API endpoint, e.g. for testing.
    pub async fn send_with_request_builder(
        request_builder: reqwest::RequestBuilder,
        creds: &Credentials,
        sender: &EmailAddress,
        msg: Message,
    ) -> MailgunResult<SendResponse> {
        let mut params = msg.to_params();
        params.insert("from".to_string(), sender.to_string());

        let res = request_builder
            .basic_auth("api", Some(creds.api_key.clone()))
            .form(&params)
            .send()
            .await?
            .error_for_status()?;

        let parsed: SendResponse = res.json().await?;
        Ok(parsed)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;

        #[ignore]
        #[tokio::test]
        async fn actually_send_email() {
            // if you want to try actually sending an email w/ your credentials add them to this test
            // and run it.
            let domain = "sandbox-some_numbers_here_probably.mailgun.org";
            let key = "something-secret-something-safe";
            let recipient = "foo@bar.com";

            let creds = Credentials::new(&key, &domain);
            let recipient = EmailAddress::address(&recipient);
            let message = Message {
                to: vec![recipient],
                subject: "Test email".to_string(),
                body: MessageBody::Text(String::from(
                    "This email is from an mailgun_v3 automated test",
                )),
                ..Default::default()
            };
            let sender =
                EmailAddress::name_address("Nick Testla", &format!("mailgun_v3@{}", &domain));

            let res = send_email(&creds, &sender, message).await;
            assert!(res.is_ok(), format!("{:?}", &res));
        }

        #[tokio::test]
        async fn test_send_with_request_builder() {
            let domain = "sandbox0123456789abcdef0123456789abcdef.mailgun.org";
            let key = "0123456789abcdef0123456789abcdef-01234567-89abcdef";
            let recipient = "user@example.com";

            let creds = Credentials::new(&key, &domain);
            let recipient = EmailAddress::address(&recipient);
            let message = Message {
                to: vec![recipient],
                subject: "Test email".to_string(),
                body: MessageBody::Text(String::from(
                    "This email is from an mailgun_v3 automated test",
                )),
                ..Default::default()
            };
            let sender =
                EmailAddress::name_address("Nick Testla", &format!("mailgun_v3@{}", &domain));

            let domain = &mockito::server_url();
            let uri = format!("/{}/{}", creds.domain, MESSAGES_ENDPOINT);

            let response = json!({
                "id": "<0123456789abcdef.0123456789abcdef@sandbox0123456789abcdef0123456789abcdef.mailgun.org>",
                "message": "Queued. Thank you."
            });
            let _m = mockito::mock("POST", uri.as_str())
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(response.to_string())
                .create();

            let url = format!("{}{}", domain, uri);
            let client = reqwest::Client::new();
            let request_builder = client.post(&url);
            let res = send_with_request_builder(request_builder, &creds, &sender, message).await;
            assert!(res.is_ok(), format!("{:?}", &res));
        }
    }
}
