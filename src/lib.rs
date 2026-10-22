use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::futures::StreamExt;
use sha2::{
    Digest, Sha256,
    digest::{array::Array, consts::U32},
};
use tokio::{fs, io::AsyncReadExt};
use tokio_stream::wrappers::ReadDirStream;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: PathBuf,
    pub hash: Hash,
}

#[derive(Debug, Clone)]
pub struct Hash(Array<u8, U32>);

impl std::fmt::LowerHex for Hash {
    fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for byte in self.0 {
            fmtr.write_fmt(format_args!("{:02x}", byte))?;
        }
        Ok(())
    }
}

impl Candidate {
    pub async fn fetch_all(path: impl AsRef<Path>, filter: &str) -> Result<Vec<Self>, Error> {
        let mut candidates = vec![];

        let read_dir = tokio::fs::read_dir(path).await?;
        let mut path_stream = ReadDirStream::new(read_dir);

        'file: while let Some(path) = path_stream.next().await {
            // ignore errors
            let Ok(path) = path else {
                continue;
            };

            let Ok(file_type) = path.file_type().await else {
                continue;
            };

            if !file_type.is_file() {
                continue;
            }

            let Ok(mut file) = fs::File::open(path.path()).await else {
                continue;
            };

            if path
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .map(str::to_lowercase)
                .is_some_and(|ext| ext == filter)
            {
                // TODO: compute parallel
                let mut hasher = Sha256::new();
                println!("start computing hash for: {:?}", path.path());
                let mut buf = [0; 4096];
                loop {
                    match file.read(&mut buf[..]).await {
                        Ok(0) => break,
                        Ok(n) => hasher.update(&buf[..n]),
                        Err(err) => {
                            eprintln!("{err:?}");
                            continue 'file;
                        }
                    }
                }
                let hash = Hash(hasher.finalize());
                println!("finished computing hash {:0x} for: {:?}", hash, path.path());

                candidates.push(Candidate {
                    path: path.path(),
                    hash,
                });
            }
        }

        Ok(candidates)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("IO operation failed: {0}")]
    Io(Arc<io::Error>),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(Arc::new(err))
    }
}
