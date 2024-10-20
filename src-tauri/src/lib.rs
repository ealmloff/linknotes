use std::{
    num::NonZero,
    sync::{Arc, OnceLock},
};

use kalosm::language::*;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::path::PathBuf;
use surrealdb::{
    engine::local::{Db, RocksDb},
    sql::Id,
    Surreal,
};

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

static DOCUMENT_TABLE: OnceLock<anyhow::Result<ContextualDocumentTable>> = OnceLock::new();
static DOCUMENT_TABLE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

static LOCATION: &str = "./.braindex";

async fn document_table() -> anyhow::Result<
    &'static DocumentTable<
        Db,
        ContextualDocument,
        Arc<CachedEmbeddingModel<Bert>>,
        DefaultSentenceChunker,
    >,
> {
    let _guard = DOCUMENT_TABLE_LOCK.lock().await;
    if DOCUMENT_TABLE.get().is_none() {
        let init = || async {
            let root = PathBuf::from(LOCATION);
            if !root.exists() {
                std::fs::create_dir_all(&root)?;
            }
            // Create database connection
            let db = Surreal::new::<RocksDb>(root.join("notes.db")).await?;

            // Select a specific namespace / database
            db.use_ns("search").use_db("documents").await?;

            // Create a table in the surreal database to store the embeddings
            let document_table = db
                .document_table_builder("documents")
                .at(root.join("documents"))
                .with_embedding_model(bert().await?.clone())
                .with_chunker(DefaultSentenceChunker)
                .build::<ContextualDocument>()
                .await?;

            anyhow::Ok(document_table)
        };

        _ = DOCUMENT_TABLE.set(init().await);
    }
    DOCUMENT_TABLE
        .get()
        .unwrap()
        .as_ref()
        .map_err(|err| anyhow::anyhow!(err))
}

#[derive(Serialize, Deserialize)]
struct ContextualDocumentLocation {
    document_id: Id,
    location: PathBuf,
    segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize)]
struct ContextualDocument {
    path: String,
    document: Document,
}

impl AsRef<Document> for ContextualDocument {
    fn as_ref(&self) -> &Document {
        &self.document
    }
}

const DOCUMENT_PATH_TABLE: &str = "document_paths";

#[derive(Serialize, Deserialize, PartialEq)]
struct Segment {
    source_char_range: Range<usize>,
}

/// Set the note with a title, contents and path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
async fn add_note(title: String, text: String, path: PathBuf) {
    let document_table = document_table().await.unwrap();
    let db = document_table.table().db();
    let path_string = path.display().to_string();
    let document = Document::from_parts(title, text);
    let chunks = DefaultSentenceChunker
        .chunk(&document, bert().await.unwrap())
        .await
        .unwrap();
    let segments = chunks
        .iter()
        .map(|chunk| Segment {
            source_char_range: chunk.byte_range.clone(),
        })
        .collect();
    // First check if the document already exists
    let current_location: Option<ContextualDocumentLocation> = db
        .select((DOCUMENT_PATH_TABLE, path_string.as_str()))
        .await
        .unwrap();
    if let Some(current_location) = &current_location {
        // If this is the same as the note already in the db, just return
        if current_location.segments == segments {
            return;
        }
        // Delete the old document if it exists
        document_table
            .delete(current_location.document_id.clone())
            .await
            .unwrap();
    }

    let contextual = ContextualDocument {
        document,
        path: path_string.clone(),
    };
    let document_id = document_table
        .insert_with_chunks(contextual, chunks)
        .await
        .unwrap();

    let location = ContextualDocumentLocation {
        document_id,
        location: path,
        segments,
    };

    // If it doesn't, create it
    if current_location.is_none() {
        let _: Option<ContextualDocumentLocation> = db
            .create((DOCUMENT_PATH_TABLE, path_string.as_str()))
            .content(location)
            .await
            .unwrap();
    // Otherwise, update it
    } else {
        let _: Option<ContextualDocumentLocation> = db
            .update((DOCUMENT_PATH_TABLE, path_string.as_str()))
            .content(location)
            .await
            .unwrap();
    }
}

#[derive(Serialize, Deserialize)]
struct SearchResult {
    distance: f32,
    path: String,
    character_range: Range<usize>,
}

/// Search for some text in the notes. Returns a list of results with the distance, path and character range of each result.
#[tauri::command]
async fn search(text: String, results: usize) -> Vec<SearchResult> {
    let document_table = document_table().await.unwrap();
    let bert = bert().await.unwrap();
    let embedding = bert.embed(text).await.unwrap();
    let nearest = document_table
        .select_nearest(embedding, results)
        .await
        .unwrap();

    nearest
        .into_iter()
        .map(|result| {
            let path = result.record.path.clone();
            let char_start = result.record.document.body()[0..result.byte_range.start]
                .chars()
                .count();
            let char_len = result.record.document.body()[result.byte_range.clone()]
                .chars()
                .count();
            let character_range = char_start..char_start + char_len;
            let distance = result.distance;
            SearchResult {
                distance,
                path,
                character_range,
            }
        })
        .collect()
}

/// Remove a note from a specific path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
async fn remove_note(path: PathBuf) {
    let document_table = document_table().await.unwrap();
    let db = document_table.table().db();
    let path_string = path.display().to_string();
    // First check if the document already exists
    let current_location: Option<ContextualDocumentLocation> = db
        .select((DOCUMENT_PATH_TABLE, &path_string))
        .await
        .unwrap();
    if let Some(current_location) = &current_location {
        // Delete the old document if it exists
        document_table
            .delete(current_location.document_id.clone())
            .await
            .unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![add_note, remove_note, search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tokio::test]
async fn test_notes() {
    // remove the index
    std::fs::remove_dir_all(LOCATION).ok();

    add_note(
        "my-note".to_string(),
        "my note is here".to_string(),
        PathBuf::from("./testing-remote-note.txt"),
    )
    .await;
    add_note(
        "my-note".to_string(),
        "my note has changed".to_string(),
        PathBuf::from("./testing-remote-note.txt"),
    )
    .await;
    remove_note(PathBuf::from("./testing-remote-note.txt")).await;

    add_note(
        "search-note".to_string(),
        "my note is here".to_string(),
        PathBuf::from("./testing-search-note.txt"),
    )
    .await;
    let results = search("my note is here".to_string(), 10).await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, "./testing-search-note.txt");
    assert_eq!(results[0].character_range, 0..15);
}
