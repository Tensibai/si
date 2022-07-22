use std::{collections::VecDeque, sync::Arc};

use async_trait::async_trait;
use dyn_clone::DynClone;
use thiserror::Error;

use super::producer::{JobProducer, JobProducerError};

pub mod faktory_processor;
pub mod test_null_processor;

#[derive(Error, Debug)]
pub enum JobQueueProcessorError {
    #[error(transparent)]
    Faktory(#[from] faktory_async::Error),
    #[error(transparent)]
    JobProducer(#[from] JobProducerError),
}

pub type JobQueueProcessorResult<T> = Result<T, JobQueueProcessorError>;

#[async_trait]
pub trait JobQueueProcessor: std::fmt::Debug + DynClone {
    async fn process_queue(
        &self,
        queue: Arc<tokio::sync::Mutex<VecDeque<Box<dyn JobProducer + Send + Sync>>>>,
    ) -> JobQueueProcessorResult<()>;
}

dyn_clone::clone_trait_object!(JobQueueProcessor);