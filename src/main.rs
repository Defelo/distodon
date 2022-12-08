use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use anyhow::Context;
use log::{debug, error, info};
use regex::Regex;
use url::Url;

use crate::discord::{check_webhook, Embed, EmbedAuthor, EmbedImage};

use self::{
    discord::{execute_webhook, WebhookPayload},
    mastodon::{fetch_posts, Id, Post},
};

mod config;
mod discord;
mod http;
mod mastodon;

#[derive(Hash)]
struct Link {
    mastodon_url: Url,
    account_id: Id,
    webhook_url: Url,
    path: PathBuf,
    last_id: Id,
}

impl Link {
    fn new(
        mastodon_url: Url,
        account_id: Id,
        webhook_url: Url,
        path: PathBuf,
    ) -> anyhow::Result<Self> {
        let last_id = Id(if path.try_exists()? {
            fs::read_to_string(&path)?.trim().parse()?
        } else {
            0
        });
        Ok(Self {
            mastodon_url,
            account_id,
            webhook_url,
            path,
            last_id,
        })
    }

    fn fetch_new_posts(&self, last_id: &Id) -> anyhow::Result<Vec<Post>> {
        fetch_posts(&self.mastodon_url, &self.account_id, last_id, 20, true)
            .context("fetching posts")
    }

    fn run(&mut self) -> anyhow::Result<()> {
        debug!("run for {:?} on {}", self.account_id, self.mastodon_url);
        let title_regex = Regex::new("<.*?>")?;
        for (post, media) in self
            .fetch_new_posts(&self.last_id)?
            .iter()
            .filter_map(|post| post.media_attachments.first().map(|ma| (post, ma)))
            .filter(|(_, media)| matches!(media.type_, mastodon::AttachmentType::Image))
            .rev()
        {
            debug!("got new post: {post:?} {media:?}");
            execute_webhook(
                self.webhook_url.clone(),
                &WebhookPayload {
                    embeds: &[&Embed {
                        title: &title_regex.replace_all(&post.content, ""),
                        timestamp: &post.created_at,
                        image: &EmbedImage { url: &media.url },
                        url: &post.url,
                        author: &EmbedAuthor {
                            name: &post.account.display_name,
                            url: &post.account.url,
                            icon_url: &post.account.avatar,
                        },
                        color: 0x595aff,
                    }],
                    username: "Mastodon",
                    avatar_url: "https://static-cdn.mastodon.social/packs/media/icons/android-chrome-512x512-ccb53c9fcbb5f61bf741cc54998318f0.png",
                },
            )?;
            self.last_id = self.last_id.max(post.id);
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    info!("loading config");
    let config = config::load_config()?;
    debug!("{config:?}");

    let path = Path::new("data");
    if !path.is_dir() {
        fs::create_dir(path)?;
    }

    let mut links = config
        .links
        .into_iter()
        .map(|link| {
            let account = mastodon::lookup_account(&link.mastodon_server_url, &link.mastodon_user)
                .context("looking up account")?;
            let mut s = DefaultHasher::new();
            (
                &link.mastodon_server_url,
                &link.mastodon_user,
                &link.webhook_url,
            )
                .hash(&mut s);
            let path = path.join(s.finish().to_string());
            check_webhook(link.webhook_url.clone()).context("checking webhook")?;
            Link::new(link.mastodon_server_url, account.id, link.webhook_url, path)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    info!("got {} links", links.len());

    loop {
        for link in &mut links {
            if let Err(err) = link.run() {
                error!("{err:#}");
            }
            if let Err(err) =
                fs::write(&link.path, link.last_id.0.to_string()).context("updating last_id")
            {
                error!("{err:#}");
            }
        }
        sleep(Duration::from_secs(config.interval));
    }
}
