use serde::Serialize;
use url::Url;

use crate::http::{self, RequestBuilderExt};

#[derive(Serialize, Debug)]
pub struct WebhookPayload<'a> {
    pub username: &'a str,
    pub avatar_url: &'a str,
    pub embeds: &'a [Embed<'a>],
}

#[derive(Serialize, Debug)]
pub struct Embed<'a> {
    pub title: String,
    pub image: EmbedImage<'a>,
    pub url: &'a Url,
    pub author: EmbedAuthor<'a>,
    pub color: u32,
    pub timestamp: &'a str,
}

#[derive(Serialize, Debug)]
pub struct EmbedImage<'a> {
    pub url: &'a Url,
}

#[derive(Serialize, Debug)]
pub struct EmbedAuthor<'a> {
    pub name: &'a str,
    pub url: &'a Url,
    pub icon_url: &'a Url,
}

pub fn execute_webhook(url: Url, payload: &WebhookPayload) -> anyhow::Result<()> {
    http::client()
        .post(url)
        .query(&[("wait", true)])
        .json(payload)
        .execute_raw()?;
    Ok(())
}

pub fn check_webhook(url: Url) -> anyhow::Result<()> {
    http::client().get(url).execute_raw()?;
    Ok(())
}
