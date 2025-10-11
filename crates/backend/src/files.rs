use std::path::{Path, PathBuf};

use entity_tag::EntityTag;
use rocket::{
    data::Capped,
    fairing::{self, Fairing, Info, Kind},
    fs::TempFile,
    tokio::io::AsyncReadExt,
    Build, Rocket, State,
};
use sha2::{Digest, Sha256};
use std::fmt::Write;

use crate::{cached_file::CachedFile, error::AppError};

pub struct FilesInitializer;

pub struct Files {
    #[cfg(not(test))]
    upload_dir: PathBuf,
    #[cfg(test)]
    upload_dir: tempfile::TempDir,
}

#[rocket::async_trait]
impl Fairing for FilesInitializer {
    fn info(&self) -> Info {
        Info {
            name: "File Manager",
            kind: Kind::Ignite,
        }
    }

    #[cfg(not(test))]
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let upload_dir: PathBuf = match rocket.figment().extract_inner("upload_dir") {
            Ok(dir) => dir,
            Err(e) => {
                error!("upload directory not specified: {}", e);
                return Err(rocket);
            }
        };

        let upload_dir = match upload_dir.canonicalize() {
            Ok(dir) => dir,
            Err(e) => {
                error!("invalid upload directory: {}", e);
                return Err(rocket);
            }
        };

        if !upload_dir.is_dir() {
            error!("given upload directory is not a directory");
            return Err(rocket);
        }

        info!("upload directory is set to {:?}", upload_dir);

        let files = Files { upload_dir };
        Ok(rocket.manage(files).mount("/uploads", routes![uploads]))
    }

    /// Handle using a temp dir for tests
    #[cfg(test)]
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let dir = tempfile::tempdir().unwrap();

        let files = Files { upload_dir: dir };
        Ok(rocket.manage(files).mount("/uploads", routes![uploads]))
    }
}

impl Files {
    pub async fn upload_file(&self, file: &mut Capped<TempFile<'_>>) -> Result<PathBuf, AppError> {
        if !file.is_complete() {
            return Err(AppError::FileTooBig(file.len()));
        }

        let mut buf = file.open().await?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 1024];

        loop {
            let count = buf.read(&mut buffer).await?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        std::mem::drop(buf);

        let hash = hasher.finalize();
        let hash: String = hash.iter().fold("".to_string(), |mut s, b| {
            write!(s, "{:02x}", b).unwrap();
            s
        });
        let extension = file.content_type().and_then(|ct| ct.extension());
        let file_path = PathBuf::from(&hash[..2]);
        std::fs::create_dir_all(self.get_path().join(&file_path))?;
        let file_name = match extension {
            Some(ext) => format!("{}.{}", hash, ext),
            None => hash,
        };
        let file_path = file_path.join(file_name);

        let dest_path = self.get_path().join(&file_path);
        if !dest_path.try_exists()? {
            // copying instead of `persist_to` because of cross-device limitations
            file.move_copy_to(dest_path).await?;
        } else if !dest_path.is_file() {
            return Err(AppError::InternalError(
                "destination path already exists, but it is not a file",
            ));
        }

        Ok(file_path)
    }

    #[cfg(not(test))]
    fn get_path(&self) -> &Path {
        self.upload_dir.as_path()
    }

    #[cfg(test)]
    fn get_path(&self) -> &Path {
        self.upload_dir.path()
    }
}

#[get("/<path..>")]
async fn uploads(path: PathBuf, files_config: &State<Files>) -> Option<CachedFile<'static>> {
    let path = files_config.get_path().join(&path);
    // Setting etag to filename as it is set to a hash of the contents on file upload.
    let etag = EntityTag::with_string(false, path.file_stem()?.to_str()?.to_owned()).ok()?;

    CachedFile::open(path, etag, chrono::Duration::weeks(1))
        .await
        .ok()
}
