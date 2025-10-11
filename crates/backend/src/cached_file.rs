use std::{fs::Metadata, io, path::Path};

use chrono::{DateTime, Utc};
use entity_tag::EntityTag;
use rocket::{fs, http::Status, response, Request, Response};

/// Wrapper around [`rocket::fs::NamedFile`] which sets appropriate headers to allow better caching.
///
/// It adds the following response headers:
/// - `etag`
/// - `last-modified`
/// - `cache-control` with a configured max age
///
/// It also responds to the `if-none-match` request header.
pub struct CachedFile<'etag> {
    file: fs::NamedFile,
    metadata: Metadata,
    etag: EntityTag<'etag>,
    max_age: chrono::Duration,
}

impl<'etag> CachedFile<'etag> {
    pub async fn open(
        path: impl AsRef<Path>,
        etag: EntityTag<'etag>,
        max_age: chrono::Duration,
    ) -> io::Result<Self> {
        let file = fs::NamedFile::open(path).await?;
        let metadata = file.metadata().await?;

        Ok(Self {
            file,
            metadata,
            etag,
            max_age,
        })
    }
}

impl<'etag, 'r> response::Responder<'r, 'r> for CachedFile<'etag> {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        if let Some(etag) = req
            .headers()
            .get("if-none-match")
            .next()
            .and_then(|raw_etag| EntityTag::from_str(raw_etag).ok())
        {
            if etag == self.etag {
                return Response::build().status(Status::NotModified).ok();
            }
        }

        let modification_time: DateTime<Utc> = self
            .metadata
            .modified()
            .expect("modification time is available on the relevant platforms")
            .into();

        Response::build_from(self.file.respond_to(req)?)
            .raw_header(
                "last-modified",
                modification_time
                    .format("%a, %d %b %Y %H:%M:%S GMT")
                    .to_string(),
            )
            .raw_header("etag", format!("{}", self.etag))
            .raw_header(
                "cache-control",
                format!("max-age={}", self.max_age.num_seconds()),
            )
            .ok()
    }
}
