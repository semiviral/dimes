use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use once_cell::sync::Lazy;
use tokio::{
    fs::File,
    sync::{Mutex, RwLock},
};
use uuid::Uuid;

static UPLOAD_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let uploads_dir = crate::TEMP_DIR.join("uploads");
    std::fs::create_dir(&uploads_dir).expect("failed to create temporary uploads directory");

    uploads_dir
});

static TEMP_UPLOADS: RwLock<BTreeMap<Uuid, Mutex<File>>> = RwLock::const_new(BTreeMap::new());

pub async fn new() -> Result<Uuid> {
    let new_uuid = Uuid::now_v7();
    let new_file = File::create(UPLOAD_DIR.join(new_uuid.to_string())).await?;
    

    let temp_uploads = TEMP_UPLOADS.write().await;

    todo!()
}
