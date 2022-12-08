use anyhow::Context;
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::Deserialize;

pub fn client() -> Client {
    Client::builder().user_agent("").build().unwrap()
}

pub trait RequestBuilderExt {
    fn execute_raw(self) -> anyhow::Result<Response>;

    fn execute<T>(self) -> anyhow::Result<T>
    where
        T: for<'de> Deserialize<'de>;
}

impl RequestBuilderExt for RequestBuilder {
    fn execute_raw(self) -> anyhow::Result<Response> {
        self.send()
            .context("sending request")?
            .error_for_status()
            .context("checking response status")
    }

    fn execute<T>(self) -> anyhow::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.execute_raw()?.json().context("parsing response")
    }
}
