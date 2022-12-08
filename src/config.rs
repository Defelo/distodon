use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
pub struct Link {
    pub mastodon_server_url: Url,
    pub mastodon_user: String,
    pub webhook_url: Url,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub interval: u64,
    pub links: Vec<Link>,
}

pub fn load_config() -> anyhow::Result<Config> {
    Ok(config::Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()?
        .try_deserialize()?)
}
