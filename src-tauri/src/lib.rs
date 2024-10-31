use kalosm::language::*;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use serde::{Deserialize, Serialize};
use slab::Slab;
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

use std::fs;

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

struct Workspace {
    location: PathBuf,
    table: OnceLock<anyhow::Result<ContextualDocumentTable>>,
    lock: tokio::sync::Mutex<()>,
}

impl Workspace {
    fn new(location: PathBuf) -> Self {
        Self {
            location,
            table: OnceLock::new(),
            lock: tokio::sync::Mutex::const_new(()),
        }
    }

    async fn files(&self) -> anyhow::Result<Vec<PathBuf>> {
        #[derive(Serialize, Deserialize)]
        struct FilePath {
            path: PathBuf,
        }

        let table = self.document_table().await?;
        let paths: Vec<FilePath> = table
            .table()
            .db()
            .query("SELECT path FROM documents")
            .await?
            .take(0)?;
        Ok(paths.into_iter().map(|p| p.path).collect())
    }

    async fn document_table(
        &self,
    ) -> anyhow::Result<
        &DocumentTable<
            Db,
            ContextualDocument,
            Arc<CachedEmbeddingModel<Bert>>,
            DefaultSentenceChunker,
        >,
    > {
        let _guard = self.lock.lock().await;
        if self.table.get().is_none() {
            let init = || async {
                let root = PathBuf::from(&self.location);
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

            _ = self.table.set(init().await);
        }
        self.table.get().unwrap().as_ref().map_err(|err| {
            let err = err.to_string();
            anyhow::anyhow!(err)
        })
    }
}

// An in memory workspace. This is an id to a global table of notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceId {
    id: usize,
}

/// This is the in memory list of open workspaces loaded by the frontend. Because we are moving between JS and Rust,
/// we need to load and unload the workspaces manually.
static OPEN_WORKSPACES: OnceLock<RwLock<Slab<Workspace>>> = OnceLock::new();

fn open_workspaces() -> &'static RwLock<Slab<Workspace>> {
    println!("open_workspaces called");
    OPEN_WORKSPACES.get_or_init(|| RwLock::new(Slab::new()))
}

/// Get a reference to a workspace by the id
fn get_workspace_ref(id: WorkspaceId) -> MappedRwLockReadGuard<'static, Workspace> {
    println!("get_workspace_ref called with id: {:?}", id);
    RwLockReadGuard::map(open_workspaces().read(), |slab| slab.get(id.id).unwrap())
}

/// Load a workspace at a path into memory. This will either load the existing workspace from the filesystem or create a new workspace at the path.
#[tauri::command]
fn load_workspace(path: PathBuf) -> WorkspaceId {
    println!("Loading workspace at {:?}", path);
    let mut workspaces = open_workspaces().write();
    let new_workspace = Workspace::new(path);
    let id = workspaces.insert(new_workspace);
    println!("Workspace loaded with id: {:?}", id);
    WorkspaceId { id }
}

#[tauri::command]
fn get_workspace_id(path: PathBuf) -> WorkspaceId {
    println!("get_workspace_id called with path: {:?}", path);
    let workspaces = open_workspaces().read();
    
    // Check if the workspace already exists
    for (id, workspace) in workspaces.iter() {
        if workspace.location == path {
            println!("Workspace found with id: {:?}", id);
            return WorkspaceId { id };
        }
    }
    
    // If not found, create a new workspace
    drop(workspaces); // Drop the read lock before acquiring a write lock
    let mut workspaces = open_workspaces().write();
    let new_workspace = Workspace::new(path.clone());
    let id = workspaces.insert(new_workspace);
    println!("New workspace created with id: {:?}", id);
    WorkspaceId { id }
}

/// Unload a workspace from memory. This should be called whenever the workspace is closed.
#[tauri::command]
fn unload_workspace(id: WorkspaceId) {
    println!("unload_workspace called with id: {:?}", id);
    let mut workspaces = open_workspaces().write();
    workspaces.remove(id.id);
    println!("Workspace unloaded with id: {:?}", id);
}

/// Permanently delete a workspace from the filesystem.
#[tauri::command]
fn delete_workspace(id: WorkspaceId) {
    println!("delete_workspace called with id: {:?}", id);
    let workspace = get_workspace_ref(id);
    let path = workspace.location.clone();
    _ = std::fs::remove_dir_all(path);
    println!("Workspace deleted with id: {:?}", id);
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

// Function to create a notes directory and save the note
#[tauri::command]
async fn add_note(title: String, text: String, workspace_id: WorkspaceId) {
    println!("Add_note called");
    println!("Workspace added with id: {:?}", workspace_id);
    
    let workspace = get_workspace_ref(workspace_id);
    let workspace_path = &workspace.location;
    let notes_dir = workspace_path.join("notes");

    // Create the notes directory if it doesn't exist
    if !notes_dir.exists() {
        fs::create_dir_all(&notes_dir).unwrap();
    }

    // Construct the file path using the title
    let file_name = format!("{}.txt", title);
    let file_path = notes_dir.join(file_name);

    // Write the note content to the file
    fs::write(&file_path, &text).unwrap();

    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();
    let path_string = file_path.display().to_string();
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
        location: file_path,
        segments,
    };

    // If it doesn't, create it
    if current_location.is_none() {
        let _: Option<ContextualDocumentLocation> = db
            .create((DOCUMENT_PATH_TABLE, path_string.as_str()))
            .content(location)
            .await
            .unwrap();
    }
    // Otherwise, update it
    else {
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
async fn search(text: String, results: usize, workspace_id: WorkspaceId) -> Vec<SearchResult> {
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.unwrap();
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
async fn remove_note(path: PathBuf, workspace_id: WorkspaceId) {
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.unwrap();
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

/// Remove a note from a specific path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
async fn read_file(path: PathBuf) -> String {
    std::fs::read_to_string(path).unwrap()
}

/// Remove a note from a specific path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
async fn files_in_workspace(workspace_id: WorkspaceId) -> Vec<PathBuf> {
    let workspace = get_workspace_ref(workspace_id);
    workspace.files().await.unwrap()
}

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
        let results = search("my note is here".to_string(), 10, workspace).await;
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
