use std::{io, sync::Arc};

mod candidated;
mod reviewed;

pub use candidated::Candidate;
pub use reviewed::Reviewed;

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
