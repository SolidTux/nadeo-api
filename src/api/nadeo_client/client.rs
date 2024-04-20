use crate::api::auth::auth_info::{AuthInfo, Service};
use crate::api::nadeo_client::client_builder::NadeoClientBuilder;
use crate::api::nadeo_request::request::NadeoRequest;
use crate::api::nadeo_request::request_builder::HttpMethod;
use reqwest::Response;

pub const NADEO_AUTH_URL: &str =
    "https://prod.trackmania.core.nadeo.online/v2/authentication/token/ubiservices";
pub const NADEO_REFRESH_URL: &str =
    "https://prod.trackmania.core.nadeo.online/v2/authentication/token/refresh";

pub const UBISOFT_APP_ID: &str = "86263886-327a-4328-ac69-527f0d20a237";

pub const EXPIRATION_TIME_BUFFER: i64 = 60;

#[derive(Debug)]
pub struct NadeoClient {
    pub client: reqwest::Client,
    pub normal_auth: AuthInfo,
    pub live_auth: AuthInfo,
}

impl NadeoClient {
    pub fn builder() -> NadeoClientBuilder {
        NadeoClientBuilder::default()
    }

    pub async fn execute(&mut self, request: NadeoRequest) -> anyhow::Result<Response> {
        match request.service {
            Service::NadeoServices => {
                if self.normal_auth.expires_in() < EXPIRATION_TIME_BUFFER {
                    self.normal_auth.refresh(&self.client).await?;
                }
            }
            Service::NadeoLiveServices => {
                if self.live_auth.expires_in() < EXPIRATION_TIME_BUFFER {
                    self.live_auth.refresh(&self.client).await?;
                }
            }
        }

        let auth_token = {
            let token = match request.service {
                Service::NadeoServices => self.normal_auth.access_token.clone(),
                Service::NadeoLiveServices => self.live_auth.access_token.clone(),
            };

            format!("nadeo_v1 t={}", token.encode())
        };

        let mut headers = request.headers;
        headers.insert("Authorization", auth_token.parse().unwrap());

        let api_request = match request.method {
            HttpMethod::Get => self.client.get(request.url),
            HttpMethod::Post => self.client.post(request.url),
        };

        Ok(api_request.headers(headers).send().await?)
    }

    pub async fn refresh_tokens(&mut self) -> anyhow::Result<()> {
        self.normal_auth.refresh(&self.client).await?;
        self.live_auth.refresh(&self.client).await?;

        Ok(())
    }
}
