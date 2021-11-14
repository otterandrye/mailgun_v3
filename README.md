# mailgun_v3

[![Build Status](https://travis-ci.org/otterandrye/mailgun_v3.svg?branch=master)](https://travis-ci.org/otterandrye/mailgun_v3)
[![Current Crates.io Version](https://img.shields.io/crates/v/mailgun_v3.svg)](https://crates.io/crates/mailgun_v3)

reqwest based web bindings for [Mailgun's v3 JSON API](https://documentation.mailgun.com/en/latest/api_reference.html)

https://docs.rs/mailgun_v3/latest/mailgun_v3/

## currently implemented

  - email send
  - email validation

## Examples

Sending an email

```rust
use mailgun_v3::email::Message;
use mailgun_v3::email::EmailAddress;
use mailgun_v3::email::MessageBody;
use mailgun_v3::Credentials;

fn main(){
    let msg = Message {
        to: vec![EmailAddress::address("target@example.org")],
        body: MessageBody::Text("hello world".to_string()),
        subject: String::from("sample subject"),
        ..Default::default()
    };
    let sender = EmailAddress::address("sender@example.org");
    let creds = Credentials::new(
        "key-abc1234567890",
        "example.org",
    );
    let res = mailgun_v3::email::send_email(&creds, &sender, msg);
    println!("{:?}", res);
}
```

More examples can be found in the examples directory.