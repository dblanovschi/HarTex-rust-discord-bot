use std::{
    error::Error
};

crate type ClientExtensionResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;
