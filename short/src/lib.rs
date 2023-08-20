//! # Short
//!
//! A small library which takes individual links and shortens them.

use std::path::PathBuf;

use thiserror::Error;
use tracing::warn;
use url::Url;
use urlencoding::encode;
use uuid::Uuid;

/// The LinkManager is a way to easily integrate `direction` into your own projects!
/// It will create a database of links, then add them to it for later recollection/editing.
struct LinkManager {
    db: sled::Db,
}

impl LinkManager {
    async fn create(db_location: Option<PathBuf>) -> Result<Self, LinkError> {
        let location = match db_location {
            Some(path) => path,
            None => {
                warn!("A database location was not provided. \
                We'll use a temporary folder, but if this is a mistake, specify a location instead. \
                Keep in mind that progress may not be saved.");

                std::env::temp_dir()
            }
        };

        Ok(LinkManager {
            db: sled::open(location)?,
        })
    }
}

/// A representation of some given link to be shortened.
/// The original link is a full URL, while the shortened link is just a "shortcut" which returns the original.
/// Aliases include all other redirected, and can include named/speciality links (if enabled) or other random links.
pub struct Link {
    identifier: Uuid,
    original_link: Url,
    shortened_link: String,
    aliases: Option<Vec<String>>,
}
#[derive(Error, Debug)]
pub enum LinkError {
    #[error("failed to parse given url")]
    InvalidLink(#[from] url::ParseError),
    #[error("failed to access database: {0}")]
    DbAccessFailure(#[from] sled::Error),
}

impl Link {
    /// Tries to generate a shorter link from a given link.
    pub async fn generate(
        link: impl AsRef<str>,
        aliases: Option<Vec<String>>,
    ) -> Result<Link, LinkError> {
        let link = link.as_ref(); // allow all kinds of strings :)
        let mut aliases = aliases.clone();

        let original_link = Url::parse(link)?;
        let identifier = Uuid::new_v4();

        // deal with aliases
        if let Some(list) = aliases {
            list.iter_mut().for_each(|s| *s = encode(&s).to_string());
        }

        // jesus christ
        tracing::debug!(
            "first element of aliases: {}",
            aliases
                .unwrap_or(vec!("no aliases given".to_string()))
                .get(0)
                .unwrap_or(&"".to_string())
        );

        Ok(Link {
            identifier,
            original_link,
            shortened_link: a,
            aliases,
        })
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[tokio::test]
    async fn try_generation() {
        // Let's try to generate 20 links, then see what comes out!
        let our_link = String::from("https://farts.google.com");
        Link::generate(our_link.clone(), None).await;
        Link::generate(&our_link, None).await;

        // how about Cow?
        let moooo = std::borrow::Cow::from("https://put.that.thang/away");
        Link::generate(moooo, None).await;
    }
}
