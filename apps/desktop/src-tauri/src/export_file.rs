use std::path::Path;

#[derive(Debug)]
pub enum ExportFileError {
    InvalidPath,
    Io(std::io::Error),
}

impl std::fmt::Display for ExportFileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => write!(formatter, "The selected export location has no file name"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ExportFileError {}

pub fn write_export_file(path: &Path, contents: &[u8]) -> Result<String, ExportFileError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .ok_or(ExportFileError::InvalidPath)?;
    std::fs::write(path, contents).map_err(ExportFileError::Io)?;
    Ok(file_name)
}
