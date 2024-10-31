use kalosm::language::*;
use note::{add_note, read_file, remove_note, ContextualDocument};
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use search::search;
use serde::{Deserialize, Serialize};
use slab::Slab;
use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use std::{
    num::NonZero,
    sync::{Arc, OnceLock},
};
use surrealdb::{
    engine::local::{Db, RocksDb},
    sql::Id,
    Surreal,
};
use workspace::{
    delete_workspace, files_in_workspace, get_workspace_id, load_workspace, unload_workspace,
};

mod note;
mod search;
mod workspace;

static BERT: OnceLock<anyhow::Result<Arc<CachedEmbeddingModel<Bert>>>> = OnceLock::new();
static BERT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn bert() -> anyhow::Result<&'static Arc<CachedEmbeddingModel<Bert>>> {
    let _guard = BERT_LOCK.lock().await;
    if BERT.get().is_none() {
        _ = BERT.set(
            Bert::new()
                .await
                .map(|e| Arc::new(e.cached(NonZero::new(2048).unwrap()))),
        );
    }
    BERT.get()
        .unwrap()
        .as_ref()
        .map_err(|err| anyhow::anyhow!(err))
}

type ContextualDocumentTable =
    DocumentTable<Db, ContextualDocument, Arc<CachedEmbeddingModel<Bert>>, DefaultSentenceChunker>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_workspace_id,
            add_note,
            remove_note,
            search,
            read_file,
            files_in_workspace,
            load_workspace,
            unload_workspace,
            delete_workspace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tokio::test]
async fn test_notes() {
    let workspace = load_workspace(PathBuf::from("./testing-workspace"));
    delete_workspace(workspace);

    for _ in 0..2 {
        add_note(
            "my-note".to_string(),
            "my note is here".to_string(),
            workspace,
        )
        .await;
        add_note(
            "my-note".to_string(),
            "my note has changed".to_string(),
            workspace,
        )
        .await;
        remove_note(PathBuf::from("./testing-remote-note.txt"), workspace).await;

        add_note(
            "search-note".to_string(),
            "my note is here".to_string(),
            workspace,
        )
        .await;
        let results = crate::search::search("my note is here".to_string(), 10, workspace).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "./testing-search-note.txt");
        assert_eq!(results[0].character_range, 0..15);

        let notes = files_in_workspace(workspace).await;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes, vec![PathBuf::from("./testing-search-note.txt")]);
    }

    delete_workspace(workspace);
    unload_workspace(workspace);
}
