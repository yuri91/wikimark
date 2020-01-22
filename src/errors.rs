use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Git error")]
    Git(#[from] git2::Error),
    #[error("Template error")]
    Tera(#[from] tera::Error),
    #[error("Serialization error")]
    Json(#[from] serde_json::Error),
}

impl warp::reject::Reject for Error {}
