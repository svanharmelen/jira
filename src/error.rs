use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Jira(#[from] goji::Error),
    #[error("missing required argument `{0}`")]
    Config(String),
}
