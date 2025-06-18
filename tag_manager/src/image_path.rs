use std::{path::PathBuf, sync::OnceLock};
use uuid::Uuid;

pub static DISCARD_PATH : OnceLock<PathBuf> = OnceLock::new();
pub static IMPORT_PATH : OnceLock<PathBuf> = OnceLock::new();
pub static STORAGE_PATH : OnceLock<PathBuf> = OnceLock::new();
pub static VIDEO_PATH : OnceLock<PathBuf> = OnceLock::new();

pub fn to_discarded() -> PathBuf {
    DISCARD_PATH.get().unwrap().join(Uuid::new_v4().to_string())
        .with_extension("png")
}

pub fn to_storage(id: u32) -> PathBuf {
    STORAGE_PATH.get().unwrap().join(id.to_string()).with_extension("png")
}

pub fn to_video(extension: &str) -> PathBuf {
    VIDEO_PATH.get().unwrap()
        .join(Uuid::new_v4().to_string())
        .with_extension(extension)
}
