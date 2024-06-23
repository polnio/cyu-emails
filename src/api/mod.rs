mod soap;

use anyhow::{bail, Context, Result};
use const_format::formatcp;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use reqwest_cookie_store::CookieStoreMutex;
use std::sync::Arc;

macro_rules! hidden_input_regex {
    ($name:literal) => {
        Regex::new(formatcp!(
            r#"<input type="hidden" name="{}" value="([^"]+)" ?\/>"#,
            $name
        ))
        .unwrap()
    };
}

macro_rules! first_match {
    ($regex:expr, $text:expr) => {
        $regex
            .captures($text)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
    };
}

const BASE_URL: &str = "https://mail.etu.cyu.fr";
const LOGIN_URL: &str =
    "https://auth.u-cergy.fr/login?service=https%3A%2F%2Fidp.u-cergy.fr%2Fidp%2FAuthn%2FRemoteUser";
const LOGIN_SAML_URL: &str = "https://sp.partage.renater.fr/ucp/Shibboleth.sso/SAML2/POST";

static LOGIN_FORM_TOKEN_REGEX: Lazy<Regex> = Lazy::new(|| hidden_input_regex!("lt"));
static LOGIN_SAML_REGEX: Lazy<Regex> = Lazy::new(|| hidden_input_regex!("SAMLResponse"));
static LOGIN_RELAY_STATE_REGEX: Lazy<Regex> = Lazy::new(|| hidden_input_regex!("RelayState"));

pub struct ApiClient {
    client: Client,
    cookie_store: Arc<CookieStoreMutex>,
}

impl ApiClient {
    pub fn new() -> Self {
        let cookie_store = Arc::new(CookieStoreMutex::default());
        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();
        Self {
            client,
            cookie_store,
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Option<String>> {
        /* self.client
        .post(formatcp!("{}{}", BASE_URL, "/soap"))
        .json(&soap::login(email, password))
        .send()
        .await
        .context("Failed to send auth request")?; */
        let response = self
            .client
            .get(BASE_URL)
            .send()
            .await
            .context("Failed to get login form")?;
        let html = response
            .text()
            .await
            .context("Failed to parse login form")?;

        let Some(token) = first_match!(LOGIN_FORM_TOKEN_REGEX, &html) else {
            bail!("Failed to retrieve form informations");
        };

        let response = self
            .client
            .post(LOGIN_URL)
            .form(&[
                ("username", username),
                ("password", password),
                ("lt", token),
                ("_eventId", "submit"),
                ("submit", "SE+CONNECTER"),
            ])
            .send()
            .await
            .context("Failed to submit login form")?;

        let text = response
            .text()
            .await
            .context("Failed to parse submit login form response")?;

        let Some(saml) = first_match!(LOGIN_SAML_REGEX, &text) else {
            bail!("Failed to retrieve SAML response");
        };

        let Some(relay_state) = first_match!(LOGIN_RELAY_STATE_REGEX, &text) else {
            bail!("Failed to retrieve Relay State");
        };

        self.client
            .post(LOGIN_SAML_URL)
            .form(&[("SAMLResponse", saml), ("RelayState", relay_state)])
            .send()
            .await
            .context("Failed to execute auth request")?;

        let token = {
            let cookie_store = self.cookie_store.lock().unwrap();
            let token = cookie_store.iter_any().find_map(|cookie| {
                (cookie.name() == "ZM_AUTH_TOKEN").then(|| cookie.value().to_string())
            });
            token
        };
        let token = token.context("Failed to retrieve auth token")?;

        Ok(Some(token))
    }
}
