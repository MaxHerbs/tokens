pub mod add;
pub mod delete;
pub mod get;
pub mod list;
pub mod logout;

use crate::config::ConfigManager;
use crate::oauth::TokenManager;
use crate::types::{ConfigFile, CredentialsProvider};
use clap::{Parser, ValueEnum};
use std::error::Error;

#[derive(Parser, Clone, Debug, ValueEnum, PartialEq)]
#[clap(rename_all = "lower")]
pub enum Format {
    Header,
}

pub struct CommandContext<'a> {
    pub config: &'a mut ConfigFile,
    pub config_manager: &'a ConfigManager,
    pub token_manager: &'a TokenManager,
    pub credentials_provider: &'a dyn CredentialsProvider,
}

#[allow(async_fn_in_trait)]
pub trait CommandHandler {
    async fn execute(&self, context: CommandContext<'_>) -> Result<(), Box<dyn Error>>;
}
