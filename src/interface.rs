use std::path::Path;

use crate::StdResult;

use async_trait::async_trait;

/// FileUri represents a file URI used to identify the file's location
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct FileUri(pub String);

impl From<FileUri> for String {
    fn from(file_uri: FileUri) -> Self {
        file_uri.0
    }
}

/// A file downloader URI
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FileDownloaderUri {
    /// A single file URI
    FileUri(FileUri),
}

impl From<String> for FileDownloaderUri {
    fn from(location: String) -> Self {
        Self::FileUri(FileUri(location))
    }
}

impl From<FileUri> for FileDownloaderUri {
    fn from(file_uri: FileUri) -> Self {
        Self::FileUri(file_uri)
    }
}

/// A file downloader
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FileDownloader: Sync + Send {
    /// Download and unpack (if necessary) a file on the disk.
    ///
    async fn download_unpack(
        &self,
        location: &FileDownloaderUri,
        file_size: u64,
        target_dir: &Path,
    ) -> StdResult<()>;
}
