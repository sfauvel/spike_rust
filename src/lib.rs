mod interface;
#[cfg(test)]
mod mock_builder;
mod retry;

/// Generic error type
type StdError = anyhow::Error;
type StdResult<T> = anyhow::Result<T, StdError>;

use interface::*;
#[cfg(test)]
use mock_builder::*;
