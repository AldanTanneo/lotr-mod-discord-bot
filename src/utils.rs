use serde_json::{Error, Value};
use serenity::model::channel::Message;

use crate::constants::MAX_JSON_FILE_SIZE;

#[derive(Debug)]
pub enum JsonMessageError {
    FileTooBig,
    DownloadError,
    JsonError(Error),
}

use JsonMessageError::*;

pub async fn get_json_from_message(msg: &Message) -> Result<Value, JsonMessageError> {
    if msg.attachments.is_empty() {
        let content = &msg.content;
        let (a, b) = (
            content.find('{').unwrap_or(0),
            content.rfind('}').unwrap_or(0),
        );
        serde_json::from_str(&content[a..=b]).map_err(|e| JsonError(e))
    } else {
        let a = &msg.attachments[0];
        if a.size <= MAX_JSON_FILE_SIZE {
            if let Ok(json_data) = a.download().await {
                serde_json::from_slice(&json_data).map_err(|e| JsonError(e))
            } else {
                Err(DownloadError)
            }
        } else {
            Err(FileTooBig)
        }
    }
}

#[macro_export]
macro_rules! success {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, ReactionType::from('✅')).await?;
    };
    ($ctx:ident, $msg:ident, $sucess_message:expr) => {
        success!($ctx, $msg);
        $msg.reply($ctx, $sucess_message).await?;
    };
}

#[macro_export]
macro_rules! failure {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, ReactionType::from('❌')).await?;
    };
    ($ctx:ident, $msg:ident, $error_message:expr) => {
        failure!($ctx, $msg);
        $msg.reply($ctx, $error_message).await?;
    };
}
