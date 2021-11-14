use mailgun_v3::email::Message;
use mailgun_v3::email::EmailAddress;
use mailgun_v3::email::MessageBody;
use mailgun_v3::Credentials;

fn main(){
    let both = Message {
        to: vec![EmailAddress::address("target@example.org")],
        body: MessageBody::Text("hello world".to_string()),
        subject: String::from("sample subject"),
        ..Default::default()
    };
    let sender = EmailAddress::address("sender@example.org");
    let creds = Credentials::with_base(
        "https://api.eu.mailgun.net/v3",
        "key-abc1234567890",
        "example.org",
    );
    let res = mailgun_v3::email::send_email(&creds, &sender, both);
    println!("{:?}", res);
}