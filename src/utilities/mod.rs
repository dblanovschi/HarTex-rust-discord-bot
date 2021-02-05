crate mod constants;
crate mod duration;

use std::{
    error::Error,
    future::Future,
    pin::Pin
};

use crate::{
    system::{
        SystemResult
    }
};

crate struct FutureResult;

impl FutureResult {
    pub async fn ok() -> SystemResult<()> {
        Ok(())
    }

    pub async fn err(error: Box<dyn Error + Send + Sync>) -> SystemResult<()> {
        Err(error)
    }

    pub async fn resolve<'asynchronous_trait>(
        result: Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>)
        -> SystemResult<()> {
        result.await
    }
}
