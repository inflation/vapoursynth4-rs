use thiserror::Error;

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("The name of the filter is invalid")]
    InvalidName,
    #[error("The filter has too many dependencies (more than `i32::MAX`)")]
    TooMuchDependency,
    #[error("Internal Error: {0}")]
    Internal(String),
}
