use std::{path::Path, sync::Arc, time::Duration, time::Instant};

use crate::StdResult;
use async_trait::async_trait;
use std::sync::Mutex;

use super::{FileDownloader, FileDownloaderUri};

/// Policy for retrying file downloads.
#[derive(Debug, PartialEq, Clone)]
pub struct FileDownloadRetryPolicy {
    /// Number of attempts to download a file.
    pub attempts: usize,
    /// Delay between two attempts.
    pub delay_between_attempts: Duration,
}

impl Default for FileDownloadRetryPolicy {
    /// Create a default retry policy.
    fn default() -> Self {
        Self {
            attempts: 3,
            delay_between_attempts: Duration::from_secs(5),
        }
    }
}

/// RetryDownloader is a wrapper around FileDownloader that retries downloading a file if it fails.
pub struct RetryDownloader {
    /// File downloader to use.
    file_downloader: Arc<dyn FileDownloader>,
    /// Number of attempts to download a file.
    retry_policy: FileDownloadRetryPolicy,
}

impl RetryDownloader {
    /// Create a new RetryDownloader.
    pub fn new(
        file_downloader: Arc<dyn FileDownloader>,
        retry_policy: FileDownloadRetryPolicy,
    ) -> Self {
        Self {
            file_downloader,
            retry_policy,
        }
    }
}

#[async_trait]
impl FileDownloader for RetryDownloader {
    async fn download_unpack(
        &self,
        location: &FileDownloaderUri,
        file_size: u64,
        target_dir: &Path,
    ) -> StdResult<()> {
        let retry_policy = &self.retry_policy;
        let mut nb_attempts = 0;
        loop {
            nb_attempts += 1;
            let start = Instant::now();
            println!("start unpack: {:?}", nb_attempts);
            let unpack = self
                .file_downloader
                .download_unpack(location, file_size, target_dir)
                .await;
            let duration = start.elapsed();
            println!("  Duration to unpack: {:?}", duration);
            let start = Instant::now();
            match unpack {
                Ok(result) => return Ok(result),
                Err(_) if nb_attempts >= retry_policy.attempts => {
                    return Err(anyhow::anyhow!(
                        "Download of location {:?} failed after {} attempts",
                        location,
                        nb_attempts
                    ));
                }
                _ => {
                    println!("  Sleep before retry: {}", nb_attempts);
                    tokio::time::sleep(retry_policy.delay_between_attempts).await
                }
            }
            let duration = start.elapsed();
            println!("  Duration during sleep: {:?}", duration);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{FileUri, MockFileDownloaderBuilder};

    use super::*;

    // #[tokio::test]
    // async fn download_return_the_result_of_download_without_retry() {
    //     let mock_file_downloader = MockFileDownloaderBuilder::default()
    //         .with_file_uri("http://whatever/00001.tar.gz")
    //         .with_compression(None)
    //         .with_success()
    //         .build();
    //     let retry_downloader = RetryDownloader::new(
    //         Arc::new(mock_file_downloader),
    //         FileDownloadRetryPolicy::never(),
    //     );

    //     retry_downloader
    //         .download_unpack(
    //             &FileDownloaderUri::FileUri(FileUri("http://whatever/00001.tar.gz".to_string())),
    //             0,
    //             Path::new("."),
    //             None,
    //             DownloadEvent::Immutable {
    //                 immutable_file_number: 1,
    //                 download_id: "download_id".to_string(),
    //             },
    //         )
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn when_download_fails_do_not_retry_by_default() {
    //     let mock_file_downloader = MockFileDownloaderBuilder::default()
    //         .with_file_uri("http://whatever/00001.tar.gz")
    //         .with_compression(None)
    //         .with_failure()
    //         .build();
    //     let retry_downloader = RetryDownloader::new(
    //         Arc::new(mock_file_downloader),
    //         FileDownloadRetryPolicy::never(),
    //     );

    //     retry_downloader
    //         .download_unpack(
    //             &FileDownloaderUri::FileUri(FileUri("http://whatever/00001.tar.gz".to_string())),
    //             0,
    //             Path::new("."),
    //             None,
    //             DownloadEvent::Immutable {
    //                 immutable_file_number: 1,
    //                 download_id: "download_id".to_string(),
    //             },
    //         )
    //         .await
    //         .expect_err("An error should be returned when download fails");
    // }

    // #[tokio::test]
    // async fn should_retry_if_fail() {
    //     let mock_file_downloader = MockFileDownloaderBuilder::default()
    //         .with_file_uri("http://whatever/00001.tar.gz")
    //         .with_compression(None)
    //         .with_failure()
    //         .with_times(2)
    //         .next_call()
    //         .with_file_uri("http://whatever/00001.tar.gz")
    //         .with_compression(None)
    //         .with_times(1)
    //         .with_success()
    //         .build();
    //     let retry_downloader = RetryDownloader::new(
    //         Arc::new(mock_file_downloader),
    //         FileDownloadRetryPolicy {
    //             attempts: 3,
    //             delay_between_attempts: Duration::from_millis(10),
    //         },
    //     );

    //     retry_downloader
    //         .download_unpack(
    //             &FileDownloaderUri::FileUri(FileUri("http://whatever/00001.tar.gz".to_string())),
    //             0,
    //             Path::new("."),
    //             None,
    //             DownloadEvent::Ancillary {
    //                 download_id: "download_id".to_string(),
    //             },
    //         )
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn should_recall_a_failing_inner_downloader_up_to_the_limit() {
    //     let mock_file_downloader = MockFileDownloaderBuilder::default()
    //         .with_file_uri("http://whatever/00001.tar.gz")
    //         .with_compression(None)
    //         .with_failure()
    //         .with_times(3)
    //         .build();
    //     let retry_downloader = RetryDownloader::new(
    //         Arc::new(mock_file_downloader),
    //         FileDownloadRetryPolicy {
    //             attempts: 3,
    //             delay_between_attempts: Duration::from_millis(10),
    //         },
    //     );

    //     retry_downloader
    //         .download_unpack(
    //             &FileDownloaderUri::FileUri(FileUri("http://whatever/00001.tar.gz".to_string())),
    //             0,
    //             Path::new("."),
    //             None,
    //             DownloadEvent::Immutable {
    //                 immutable_file_number: 1,
    //                 download_id: "download_id".to_string(),
    //             },
    //         )
    //         .await
    //         .expect_err("An error should be returned when all download attempts fail");
    // }

    #[tokio::test]
    async fn should_delay_between_retries() {
        let mock_file_downloader = MockFileDownloaderBuilder::default()
            .with_file_uri("http://whatever/00001.tar.gz")
            .with_failure()
            .with_times(4)
            .build();
        let delay = Duration::from_millis(50);
        let retry_downloader = RetryDownloader::new(
            Arc::new(mock_file_downloader),
            FileDownloadRetryPolicy {
                attempts: 4,
                delay_between_attempts: delay,
            },
        );

        let start = Instant::now();
        retry_downloader
            .download_unpack(
                &FileDownloaderUri::FileUri(FileUri("http://whatever/00001.tar.gz".to_string())),
                0,
                Path::new("."),
            )
            .await
            .expect_err("An error should be returned when all download attempts fail");
        let duration = start.elapsed();

        println!("Duration: {:?}", duration);
        assert!(
            duration >= delay * 3,
            "Duration should be at least 3 times the delay ({}ms) but was {}ms",
            delay.as_millis() * 3,
            duration.as_millis()
        );
        assert!(
            duration < delay * 4,
            "Duration should be less than 4 times the delay ({}ms) but was {}ms",
            delay.as_millis() * 4,
            duration.as_millis()
        );
    }

    #[tokio::test]
    async fn compute_time_when_sleeping() {
        // https://github.com/tokio-rs/tokio/discussions/4473
        for time_to_sleep in [
            Duration::from_millis(5),
            Duration::from_millis(50),
            Duration::from_millis(100),
        ] {
            // let time_to_sleep = Duration::from_millis(50);
            for i in 0..10 {
                let start = Instant::now();
                tokio::time::sleep(time_to_sleep).await;
                let duration = start.elapsed();
                println!(
                    "Duration during sleep ({}): {:?}",
                    time_to_sleep.as_millis(),
                    duration
                );
            }
        }
    }

    #[tokio::test]
    async fn compute_time_when_interval_waiting() {
        // https://github.com/tokio-rs/tokio/discussions/4473
        for time_to_sleep in [
            Duration::from_millis(5),
            Duration::from_millis(50),
            Duration::from_millis(100),
        ] {
            let mut interval = tokio::time::interval(time_to_sleep);
            // let time_to_sleep = Duration::from_millis(50);
            for i in 0..10 {
                let start = Instant::now();
                // tokio::time::sleep(time_to_sleep).await;
                interval.tick().await;
                let duration = start.elapsed();
                println!(
                    "Duration during tick ({}): {:?}",
                    time_to_sleep.as_millis(),
                    duration
                );
            }
        }
    }

    struct MyFileDownloader {
        expected_delay_between_retry: Duration,
        nb_attempts: Mutex<i32>,
        last_start_attempt: Mutex<Option<Instant>>,
    }

    impl MyFileDownloader {
        fn new(expected_delay_between_retry: Duration) -> Self {
            Self {
                expected_delay_between_retry,
                nb_attempts: Mutex::new(0),
                last_start_attempt: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl FileDownloader for MyFileDownloader {
        async fn download_unpack(
            &self,
            _location: &FileDownloaderUri,
            _file_size: u64,
            _target_dir: &Path,
        ) -> StdResult<()> {
            let mut last_start_attempt = self.last_start_attempt.lock().unwrap();
            if let Some(last_start_attempt) = *last_start_attempt {
                let duration = last_start_attempt.elapsed();
                assert!(duration >= self.expected_delay_between_retry);
                assert!(duration < 2 * self.expected_delay_between_retry);
                println!("  Duration to unpack in MyFileDownloader: {:?}", duration);
            }
            *last_start_attempt = Some(Instant::now());

            let mut attempts = self.nb_attempts.lock().unwrap();
            *attempts += 1;

            Err(anyhow::anyhow!("Download failed"))
        }
    }

    #[tokio::test]
    async fn should_delay_between_retries_with_my_file_downloader() {
        let delay = Duration::from_millis(50);
        let mock_file_downloader = Arc::new(MyFileDownloader::new(delay));

        let retry_downloader = RetryDownloader::new(
            mock_file_downloader.clone(),
            FileDownloadRetryPolicy {
                attempts: 4,
                delay_between_attempts: delay,
            },
        );

        retry_downloader
            .download_unpack(
                &FileDownloaderUri::FileUri(FileUri("http://whatever/00001.tar.gz".to_string())),
                0,
                Path::new("."),
            )
            .await
            .expect_err("An error should be returned when all download attempts fail");

        assert_eq!(4, *mock_file_downloader.nb_attempts.lock().unwrap());
    }
}

// https://docs.rs/tokio/latest/tokio/time/fn.interval.html
