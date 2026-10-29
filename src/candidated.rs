use std::{
    ffi::OsStr,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use iced::futures::StreamExt;
use sha2::{
    Digest, Sha256,
    digest::{array::Array, consts::U32},
};
use tokio::task;
use tokio_stream::wrappers::ReadDirStream;

use crate::Error;

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

        while let Some(path) = path_stream.next().await {
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

            if path
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .map(str::to_lowercase)
                .is_some_and(|ext| ext == filter)
            {
                let path = path.path();
                let Ok(Some((path, hash))) = task::spawn_blocking(move || {
                    let mut hasher = Sha256::new();

                    let mut file = fs::File::open(&path).ok()?;
                    let mut buf = [0; 4096];

                    println!("start computing hash for: {:?}", path);
                    loop {
                        match file.read(&mut buf[..]) {
                            Ok(0) => break,
                            Ok(n) => hasher.update(&buf[..n]),
                            Err(err) => {
                                eprintln!("{err:?}");
                                return None;
                            }
                        }
                    }
                    let hash = Hash(hasher.finalize());
                    println!("finished computing hash {:0x} for: {:?}", hash, path);

                    Some((path, hash))
                })
                .await
                else {
                    continue;
                };

                candidates.push(Candidate { path, hash });
            }
        }

        Ok(candidates)
    }
}
