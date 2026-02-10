use aws_sdk_s3::primitives::ByteStream;
use rocket::{
    data::Capped,
    fairing::{self, Fairing, Info, Kind},
    fs::TempFile,
    tokio::io::AsyncReadExt,
    Build, Rocket,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fmt::Write;

use crate::error::AppError;

pub struct FilesInitializer;

#[derive(Deserialize)]
struct S3Config {
    url: String,
    bucket: String,
    #[serde(default)]
    use_mock: bool,
}

pub struct Files {
    s3_client: aws_sdk_s3::Client,
    s3_config: S3Config,
}

#[rocket::async_trait]
impl Fairing for FilesInitializer {
    fn info(&self) -> Info {
        Info {
            name: "File Manager",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let s3_config: S3Config = match rocket.figment().focus("s3").extract() {
            Ok(dir) => dir,
            Err(e) => {
                error!("s3 configuration incomplete: {}", e);
                return Err(rocket);
            }
        };

        let mut builder = aws_config::load_defaults(aws_config::BehaviorVersion::latest())
            .await
            .into_builder()
            .region(aws_config::Region::new("eu-west-1"));
        // For some stupid reason it isn't possible to conditionally set the endpoint url without a
        // mutable reference...
        builder.set_endpoint_url(s3_config.use_mock.then(|| s3_config.url.clone()));

        let config = builder.build();
        let config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(s3_config.use_mock)
            .build();

        let s3_client = aws_sdk_s3::Client::from_conf(config);

        let files = Files {
            s3_client,
            s3_config,
        };
        Ok(rocket.manage(files))
    }
}

impl Files {
    pub async fn upload_file(&self, file: &mut Capped<TempFile<'_>>) -> Result<String, AppError> {
        if !file.is_complete() {
            return Err(AppError::FileTooBig(file.len()));
        }

        let mut content = Vec::new();
        file.open().await?.read_to_end(&mut content).await?;

        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            hasher.finalize().iter().fold("".to_string(), |mut s, b| {
                write!(s, "{:02x}", b).unwrap();
                s
            })
        };

        let key = if let Some(ext) = file.content_type().and_then(|ct| ct.extension()) {
            hash + "." + ext.as_str()
        } else {
            hash
        };

        self.s3_client
            .put_object()
            .bucket(&self.s3_config.bucket)
            .key(&key)
            .body(ByteStream::from(content))
            .set_content_type(
                file.content_type()
                    .map(|content_type| content_type.to_string()),
            )
            .send()
            .await?;

        Ok(key)
    }

    pub fn file_url(&self, key: &str) -> String {
        format!("{}/{}/{}", self.s3_config.url, self.s3_config.bucket, key)
    }
}
