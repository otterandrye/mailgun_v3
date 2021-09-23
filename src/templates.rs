use std::collections::HashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use crate::{Credentials, MailgunResult};

const TEMPLATES_ENDPOINT: &str = "templates";
const TEMPLATE_VERSIONS_ENDPOINT: &str = "versions";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateResponse {
    pub message: String,
    pub template: TemplateResponse,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetTemplatesResponse {
    pub items: Vec<TemplateResponse>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetSingleTemplateResponse {
    pub template: TemplateResponse,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateResponse {
    pub created_at: String,
    pub created_by: String,
    pub description: String,
    pub name: String,
    pub id: String,
    pub version: Option<VersionResponse>,
    pub versions: Option<Vec<VersionResponse>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    created_at: String,
    engine: String,
    tag: String,
    comment: String,
    mjml: String,
    template: Option<String>,
    id: Option<String>,
    active: bool,
}

#[derive(Serialize, Default, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub name: String,
    pub description: String,
    pub template: Option<String>,
    pub tag: Option<String>,
    pub engine: Option<String>,
    pub comment: Option<String>,
}

impl Template {
    fn into_params(self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        params.insert("name".to_string(), self.name);
        params.insert("description".to_string(), self.description);

        if let Some(template) = self.template {
            params.insert("template".to_string(), template);
        }

        if let Some(tag) = self.tag {
            params.insert("tag".to_string(), tag);
        }

        if let Some(engine) = self.engine {
            params.insert("engine".to_string(), engine);
        }

        if let Some(comment) = self.comment {
            params.insert("comment".to_string(), comment);
        }

        params
    }
}

//------------------------------
pub async fn create_template(
    creds: &Credentials,
    template: Template,
) -> MailgunResult<CreateTemplateResponse> {
    let client = reqwest::Client::new();
    create_template_with_client(&client, creds, template).await
}

pub async fn create_template_with_client(
    client: &reqwest::Client,
    creds: &Credentials,
    template: Template,
) -> MailgunResult<CreateTemplateResponse> {
    let url = format!("{}/{}/{}", creds.api_base, creds.domain, TEMPLATES_ENDPOINT);
    let request_builder = client.post(&url);
    create_template_with_request_builder(request_builder, creds, template).await
}

pub async fn create_template_with_request_builder(
    request_builder: reqwest::RequestBuilder,
    creds: &Credentials,
    template: Template,
) -> MailgunResult<CreateTemplateResponse> {
    let params = template.into_params();

    let res = request_builder
        .basic_auth("api", Some(creds.api_key.clone()))
        .form(&params)
        .send()
        .await?
        .error_for_status()?;

    let parsed: CreateTemplateResponse = res.json().await?;
    Ok(parsed)
}
//------------------------------

pub async fn get_templates(
    creds: &Credentials,
    template_name: Option<String>,
    do_fetch_versions: bool,
) -> MailgunResult<GetTemplatesResponse> {
    let client = reqwest::Client::new();
    get_templates_with_client(&client, creds, template_name, do_fetch_versions).await
}

pub async fn get_templates_with_client(
    client: &reqwest::Client,
    creds: &Credentials,
    template_name: Option<String>,
    do_fetch_versions: bool,
) -> MailgunResult<GetTemplatesResponse> {
    let url = if let Some(template_name) = template_name.clone() {
        if do_fetch_versions {
            format!("{}/{}/{}/{}/{}", creds.api_base, creds.domain, TEMPLATES_ENDPOINT, template_name, TEMPLATE_VERSIONS_ENDPOINT)
        } else {
            format!("{}/{}/{}/{}", creds.api_base, creds.domain, TEMPLATES_ENDPOINT, template_name)
        }
    } else {
        format!("{}/{}/{}", creds.api_base, creds.domain, TEMPLATES_ENDPOINT)
    };
    let request_builder = client.get(&url);
    get_templates_with_request_builder(request_builder, creds, template_name).await
}

pub async fn get_templates_with_request_builder(
    request_builder: reqwest::RequestBuilder,
    creds: &Credentials,
    template_name: Option<String>,
) -> MailgunResult<GetTemplatesResponse> {
    let res = request_builder
        .basic_auth("api", Some(creds.api_key.clone()))
        .send()
        .await?
        .error_for_status()?;

    let response = if template_name.is_some() {
        let parsed: GetSingleTemplateResponse = res.json().await?;
        GetTemplatesResponse {
            items: vec![parsed.template]
        }
    } else {
        let parsed: GetTemplatesResponse = res.json().await?;
        parsed
    };
    
    Ok(response)
}


#[cfg(test)]
mod tests {
    use super::*;

    const DOMAIN: &str = "xxxxxxxxx.mailgun.org";
    const KEY: &str = "xxxxxxxx-yyyyyyyyy-zzzzzzzz";

    #[tokio::test]
    async fn test_create_template() {
        let creds = Credentials::new(&KEY, &DOMAIN);
        let template = Template {
            name: "mytemplate12".to_string(),
            description: "template description12".to_string(),
            template: Some("{{fname}} {{lname}}".to_string()),
            tag: None,
            engine: Some("handlebars".to_string()),
            comment: None,
        };
        let res = create_template(&creds, template).await;
        println!("response = {:?}", &res);
    }

    #[tokio::test]
    async fn test_get_all_templates() {
        let creds = Credentials::new(&KEY, &DOMAIN);
        let res = get_templates(&creds, None, false).await;
        println!("response = {:?}", &res);
    }

    #[tokio::test]
    async fn test_get_single_template_no_version() {
        let creds = Credentials::new(&KEY, &DOMAIN);
        let res = get_templates(&creds, Some("mytemplate12".to_string()), false).await;
        println!("response = {:?}", &res);
    }

    #[tokio::test]
    async fn test_get_single_template_all_versions() {
        let creds = Credentials::new(&KEY, &DOMAIN);
        let res = get_templates(&creds, Some("mytemplate12".to_string()), true).await;
        println!("response = {:?}", &res);
    }

}