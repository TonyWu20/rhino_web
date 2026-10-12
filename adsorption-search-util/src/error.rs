use std::fmt::Display;

#[derive(Debug)]
pub enum LoadError {
    UnsupportedFormat(String),
    ParseFailure,
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::UnsupportedFormat(ext) => write!(f, "Unsupported format: {ext}"),
            LoadError::ParseFailure => {
                write!(f, "Failed to parse the cell content")
            }
        }
    }
}
