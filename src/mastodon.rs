use anyhow::Context;
use log::debug;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use url::Url;

use crate::http::{self, RequestBuilderExt};

pub fn lookup_account(base_url: &Url, name: &str) -> anyhow::Result<Account> {
    debug!("fetching account info for {name} on {base_url}");
    http::client()
        .get(
            base_url
                .join("api/v1/accounts/lookup")
                .context("creating url")?,
        )
        .query(&[("acct", name)])
        .execute()
        .context("fetching")
}

pub fn fetch_posts(
    base_url: &Url,
    account_id: &Id,
    min_id: &Id,
    limit: u16,
    exclude_reblogs: bool,
) -> anyhow::Result<Vec<Post>> {
    http::client()
        .get(
            base_url
                .join(&format!("api/v1/accounts/{}/statuses", account_id.0))
                .context("creating url")?,
        )
        .query(&[("min_id", min_id.0)])
        .query(&[("limit", limit)])
        .query(&[("exclude_reblogs", exclude_reblogs)])
        .execute()
        .context("fetching")
}

#[derive(Deserialize, Debug)]
pub struct Post {
    pub id: Id,
    pub url: Url,
    pub created_at: String,
    pub sensitive: bool,
    pub content: String,
    pub media_attachments: Vec<MediaAttachment>,
    pub account: Account,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: Id,
    pub username: String,
    pub display_name: String,
    pub url: Url,
    pub avatar: Url,
}

#[derive(Deserialize, Debug)]
pub struct MediaAttachment {
    pub id: Id,
    #[serde(rename = "type")]
    pub type_: AttachmentType,
    pub url: Url,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    Unknown,
    Image,
    Gifv,
    Video,
    Audio,
}

#[derive(Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct Id(#[serde(deserialize_with = "deserialize_number_from_string")] pub u64);
