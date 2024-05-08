use std::fmt::Display;

#[derive(Debug)]
pub enum LoadError {
    UnsupportedFormat(String),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::UnsupportedFormat(ext) => write!(f, "Unsupported format: {}", ext),
        }
    }
}
