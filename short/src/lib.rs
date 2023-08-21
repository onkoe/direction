//! # Short
//!
//! A small library which takes individual links and shortens them.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::spawn_blocking;
use tracing::{info, instrument, warn};
use url::Url;
use uuid::Uuid;

/// The LinkManager is a way to easily integrate `direction` into your own projects!
/// It will create a database of links, then add them to it for later recollection/editing.
#[derive(Debug, Clone)]
pub struct LinkManager {
    db: sled::Db,
}

impl LinkManager {
    /// Attempts to create a new LinkManager.
    /// The database location is optional. If you choose to omit it, a temporary database will be created for you using `std::env::temp_dir()`.
    #[allow(clippy::redundant_closure)] // remove spawn_blocking() warns
    #[instrument]
    pub async fn create(db_location: Option<PathBuf>) -> Result<Self, LinkError> {
        let location = match db_location {
            Some(path) => path,
            None => {
                warn!("A database location was not provided. \
                We'll use a temporary folder, but if this is a mistake, specify a location instead. \
                Keep in mind that progress may not be saved.");

                spawn_blocking(|| std::env::temp_dir()).await?
            }
        };

        let location_clone = location.clone();
        let db: sled::Db = spawn_blocking(move || sled::open(location_clone)).await??;

        info!(
            "A LinkManager has been created or accessed at the following location: {}",
            location.display()
        );

        Ok(LinkManager { db })
    }

    /// Tries to generate a shorter link from a given link.
    #[instrument(skip(link))]
    pub async fn generate_link(
        &mut self,
        link: impl AsRef<str>,
        aliases: Option<Vec<String>>,
    ) -> Result<Link, LinkError> {
        let link = link.as_ref(); // allow all kinds of strings :)

        let original_link = Url::parse(link)?;
        let identifier = Uuid::new_v4();

        // deal with aliases
        let aliases = aliases.map(|list| {
            list.iter()
                .map(|s| urlencoding::encode(s).to_string())
                .collect()
        });

        // TODO: actually make links shorten!
        let shortened_link = "farts".into();

        let encapsulated_link = Link {
            identifier,
            original_link,
            shortened_link,
            aliases,
        };

        // add it to the db
        // TODO: use aliases, too!
        self.db.insert(
            encapsulated_link.shortened_link.clone(),
            encapsulated_link.clone().encode().await?,
        )?;

        Ok(encapsulated_link)
    }

    /// Attempts to find a link in the database - given its shortened form.
    pub async fn resolve_link(&self, short_link: impl AsRef<str>) -> Result<Link, LinkError> {
        let short_link = short_link.as_ref();

        match self.db.get(short_link)? {
            Some(ivec) => Ok(bincode::deserialize(ivec.to_vec().as_slice())?),
            None => Err(LinkError::LinkNotFound(short_link.into())),
        }
    }
}

/// A representation of some given link to be shortened.
/// The original link is a full URL, while the shortened link is just a "shortcut" which returns the original.
/// Aliases include all other redirected, and can include named/speciality links (if enabled) or other random links.
#[allow(unused)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    identifier: Uuid,
    original_link: Url,
    shortened_link: String,
    aliases: Option<Vec<String>>,
}

impl Link {
    /// Encodes the link as a vector of bytes.
    /// This allows it to be stored in a database.
    async fn encode(self) -> Result<Vec<u8>, LinkError> {
        Ok(bincode::serialize(&self)?)
    }
}

/// An error that occurs when handling links.
#[derive(Debug, Error)]
pub enum LinkError {
    #[error("failed to parse given url")]
    InvalidLink(#[from] url::ParseError),
    #[error("failed to access database: {0}")]
    DbAccessFailure(#[from] sled::Error),
    #[error("encoding of link to db failed: {0}")]
    LinkEncodingFailure(#[from] bincode::Error),
    #[error("link not found: {0}")]
    LinkNotFound(String),
    #[error("internal error. blocking operation failed to join: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_test::traced_test;

    #[allow(unused)]
    use super::*;

    #[tokio::test]
    #[traced_test]
    async fn try_generation() {
        #![allow(unused_must_use)]
        tracing_subscriber::fmt::fmt()
            .with_max_level(Level::TRACE)
            .finish();

        let mut lm = LinkManager::create(None).await.unwrap();

        // Let's try to generate 20 links, then see what comes out!
        let our_link = String::from("https://farts.google.com");

        lm.generate_link(our_link.clone(), None).await;
        lm.generate_link(&our_link, None).await;

        // how about Cow?
        let moooo = std::borrow::Cow::from("https://put.that.thang/away");
        lm.generate_link(moooo, None).await;
    }
}
