use kalosm::language::*;
use note::{get_tags, read_note, remove_note, save_note, set_tags, ContextualDocument};
use search::{context_search, search};
use std::{
    num::NonZero,
    sync::{Arc, OnceLock},
};
use surrealdb::engine::local::Db;
use workspace::{
    delete_workspace, files_in_workspace, get_workspace_id, load_workspace, unload_workspace,
};

#[cfg(test)]
use pretty_assertions::assert_eq;

mod classifier;
mod note;
mod search;
mod workspace;

static BERT: OnceLock<anyhow::Result<Arc<CachedEmbeddingModel<Bert>>>> = OnceLock::new();
static BERT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn bert() -> anyhow::Result<&'static Arc<CachedEmbeddingModel<Bert>>> {
    let _guard = BERT_LOCK.lock().await;
    if BERT.get().is_none() {
        _ = BERT.set(
            Bert::builder()
                .with_source(BertSource::snowflake_arctic_embed_small())
                .build()
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
            save_note,
            set_tags,
            get_tags,
            remove_note,
            search,
            read_note,
            files_in_workspace,
            load_workspace,
            unload_workspace,
            delete_workspace,
            context_search
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tokio::test]
async fn test_notes() {
    _ = tracing_subscriber::fmt::try_init();

    let temp = std::env::temp_dir();
    let workspace_path = temp.join("testing-notes-workspace");
    _ = std::fs::remove_dir_all(&workspace_path);
    let workspace = load_workspace(workspace_path);

    save_note(
        "mynote".to_string(),
        "my note is here".to_string(),
        workspace,
    )
    .await
    .unwrap();
    remove_note("mynote".to_string(), workspace).await;

    save_note(
        "search-note".to_string(),
        "my note is here".to_string(),
        workspace,
    )
    .await
    .unwrap();
    let results = crate::search::search("my note is here".to_string(), Vec::new(), 10, workspace)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "search-note");
    assert_eq!(results[0].character_range, 0..15);

    let notes = files_in_workspace(workspace).await;
    assert_eq!(notes.len(), 1);
    assert_eq!(
        notes[0].document,
        Document::from_parts("search-note".to_string(), "my note is here".to_string())
    );
    assert_eq!(notes[0].tags.len(), 1);
    assert_eq!(notes[0].tags[0].manual, false);
    assert!(["Math", "Computer Science"].contains(&notes[0].tags[0].name.as_str()));

    delete_workspace(workspace);
    unload_workspace(workspace);
}
