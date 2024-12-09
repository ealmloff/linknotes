/*!
# Prologue Comments
## Name of Code Artifact: Workspace Application Framework

## Brief Description: This code provides a framework for managing workspaces and notes in the application.
## Programmer’s Name: Evan Almoff

## Date Created: 2024-10-14
## Dates Revised and Description of Revisions: 
2024-10-14: Initial creation of the workspace module.
2024-10-15: Added workspace loading and unloading functions.
2024-10-16: Implemented note management functions.


## Preconditions: A Tauri application context is required for the run function.
The BERT model must be initialized using the bert function before using search functionalities dependent on embeddings.
Workspaces must be loaded for note-related operations.
Acceptable and Unacceptable Input Values/Types:

## Postconditions:

### Searches return relevant results based on embeddings or fail with descriptive errors.

## Return Values/Types:

### Functions return Result types with success yielding appropriate outputs (e.g., workspaceID).
### Errors are encapsulated in the anyhow::Result type for robust error handling.
### Error and Exception Condition Values/Types: anyhow::Error: 
-> For general errors. 
-> Initialization errors for BERT or issues with workspace paths are raised.
-> File I/O errors occur when accessing or modifying workspace files.
## Side Effects:
### Workspace files may be created, modified, or deleted based on the function.
### A global singleton (BERT) is initialized, consuming system resources.

## Invariants:

### Once initialized, the BERT model remains immutable.
### Workspace states should remain consistent after note operations.

## Known Faults:

### Potential performance bottleneck during embedding generation if BERT initialization is delayed.
### Edge cases with workspace paths or malformed input data may cause unexpected behavior.

*/


use kalosm::language::*;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use serde::{Deserialize, Serialize};
use slab::Slab;
use std::path::PathBuf;
use std::sync::OnceLock;
use surrealdb::{engine::local::RocksDb, Surreal};

use crate::classifier::TagClassifier;
use crate::note::{ContextualDocument, Tag};
use crate::{bert, ContextualDocumentTable};

pub struct Workspace {
    pub location: PathBuf,
    table: OnceLock<anyhow::Result<ContextualDocumentTable>>,
    tags: RwLock<Vec<String>>,
    classifier: RwLock<Option<TagClassifier>>,
    lock: tokio::sync::Mutex<()>,
}

/// Represents a workspace that manages documents, tags, and a classifier.
///
/// # Fields
/// - `location`: The file path where the workspace is located.
/// - `table`: A lock for the document table, initialized once.
/// - `lock`: A mutex lock for synchronizing access to the document table.
/// - `tags`: A read-write lock for managing tags associated with documents.
/// - `classifier`: A read-write lock for the document classifier.
///
/// # Methods
/// - `new(location: PathBuf) -> Self`: Creates a new workspace at the specified location.
/// - `document_path(&self, title: &str) -> anyhow::Result<PathBuf>`: Returns the file path for a document with the given title, creating the notes directory if it doesn't exist.
/// - `files(&self) -> anyhow::Result<Vec<ContextualDocument>>`: Asynchronously retrieves all contextual documents from the document table.
/// - `document_table(&self) -> anyhow::Result<&ContextualDocumentTable>`: Asynchronously initializes and returns the document table, creating the database connection and table if necessary.
/// - `get_tag_id(&self, tag: &str) -> u32`: Returns the ID of the specified tag, adding it to the tag list if it doesn't exist.
/// - `get_tag_name(&self, id: u32) -> String`: Returns the name of the tag with the specified ID.
/// - `tag_count(&self) -> usize`: Returns the number of tags in the workspace.
/// - `retrain_classifier(&self)`: Retrains the document classifier by clearing the current classifier.
/// - `classify(&self, document: &ContextualDocument) -> anyhow::Result<Tag>`: Asynchronously classifies the given document, initializing the classifier if necessary.
impl Workspace {
    fn new(location: PathBuf) -> Self { // Create a new workspace at the specified location
        Self {
            location,
            table: OnceLock::new(),
            lock: tokio::sync::Mutex::const_new(()),
            tags: RwLock::new(Vec::new()),
            classifier: RwLock::new(None),
        }
    }

    pub fn document_path(&self, title: &str) -> anyhow::Result<PathBuf> { // Returns the file path for a document with the given title
        let notes_dir = self.location.join("notes");

        // Create the notes directory if it doesn't exist
        if !notes_dir.exists() {
            std::fs::create_dir_all(&notes_dir).unwrap();
        }

        // Construct the file path using the title
        let file_name = format!("{}.txt", title);
        let file_path = notes_dir.join(file_name);
        Ok(file_path)
    }

    async fn files(&self) -> anyhow::Result<Vec<ContextualDocument>> { // Asynchronously retrieves all contextual documents from the document table
        #[derive(Serialize, Deserialize)]
        struct FilePath {
            path: PathBuf,
        }

        let table = self.document_table().await?;
        let paths: Vec<ContextualDocument> =
            table.table().db().select(table.table().table()).await?;
        Ok(paths)
    }

    pub async fn document_table(&self) -> anyhow::Result<&ContextualDocumentTable> { // Asynchronously initializes and returns the document table
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
        self.table.get().unwrap().as_ref().map_err(|err| { // Return an error if the document table is not initialized
            let err = err.to_string();
            anyhow::anyhow!(err)
        })
    }

    pub fn get_tag_id(&self, tag: &str) -> u32 { // Returns the ID of the specified tag, adding it to the tag list if it doesn't exist
        let mut tags_mut = self.tags.write();
        match tags_mut.iter().position(|t| t == tag) { // Return the ID of the specified tag
            Some(index) => index as u32,
            None => {
                let index = tags_mut.len() as u32;
                tags_mut.push(tag.to_string());
                index
            }
        }
    }

    pub fn get_tag_name(&self, id: u32) -> String { // Returns the name of the tag with the specified ID
        let tag_read = self.tags.read();
        tag_read[id as usize].clone()
    }

    pub fn tag_count(&self) -> usize {
        let tag_read = self.tags.read();
        tag_read.len()
    }

    pub fn retrain_classifier(&self) {
        let mut classifier_mut = self.classifier.write();
        classifier_mut.take();
    }

    pub async fn classify(&self, document: &ContextualDocument) -> anyhow::Result<Tag> { // Asynchronously classifies the given document
        let mut classifier_mut = self.classifier.write();
        let mut classifier = classifier_mut.take();
        if classifier.is_none() {
            let document_table = self.document_table().await?;
            let documents = document_table.table().select_all().await?;
            classifier = Some(TagClassifier::new(self, &documents, |_| {}).await?);
        }
        let classifier = classifier.unwrap();
        let tag = classifier.classify(self, document).await?;
        *classifier_mut = Some(classifier);
        Ok(tag)
    }
}

// An in memory workspace. This is an id to a global table of notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)] // Represents a workspace ID
pub struct WorkspaceId {
    id: usize,
}

/// This is the in memory list of open workspaces loaded by the frontend. Because we are moving between JS and Rust,
/// we need to load and unload the workspaces manually.
static OPEN_WORKSPACES: OnceLock<RwLock<Slab<Workspace>>> = OnceLock::new();

fn open_workspaces() -> &'static RwLock<Slab<Workspace>> { // Get a reference to the open workspaces
    tracing::info!("open_workspaces called");
    OPEN_WORKSPACES.get_or_init(|| RwLock::new(Slab::new()))
}

/// Get a reference to a workspace by the id
pub fn get_workspace_ref(id: WorkspaceId) -> MappedRwLockReadGuard<'static, Workspace> { // Get a reference to a workspace by the ID
    tracing::info!("get_workspace_ref called with id: {:?}", id);
    RwLockReadGuard::map(open_workspaces().read(), |slab| slab.get(id.id).unwrap())
}

/// Load a workspace at a path into memory. This will either load the existing workspace from the filesystem or create a new workspace at the path.
#[tauri::command]
pub fn load_workspace(path: PathBuf) -> WorkspaceId { // Load a workspace at a path into memory
    tracing::info!("Loading workspace at {:?}", path);
    let mut workspaces = open_workspaces().write();
    let new_workspace = Workspace::new(path);
    let id = workspaces.insert(new_workspace);
    tracing::info!("Workspace loaded with id: {:?}", id);
    WorkspaceId { id }
}

#[tauri::command]
pub fn get_workspace_id(path: PathBuf) -> WorkspaceId { // Get the ID of a workspace at a path
    tracing::info!("get_workspace_id called with path: {:?}", path);
    let workspaces = open_workspaces().read();

    // Check if the workspace already exists
    for (id, workspace) in workspaces.iter() {
        if workspace.location == path {
            tracing::info!("Workspace found with id: {:?}", id);
            return WorkspaceId { id };
        }
    }

    // If not found, create a new workspace
    drop(workspaces); // Drop the read lock before acquiring a write lock
    let mut workspaces = open_workspaces().write();
    let new_workspace = Workspace::new(path.clone());
    let id = workspaces.insert(new_workspace);
    tracing::info!("New workspace created with id: {:?}", id);
    WorkspaceId { id }
}

// Unload a workspace from memory. This should be called whenever the workspace is closed.
#[tauri::command]
pub fn unload_workspace(id: WorkspaceId) { 
    tracing::info!("unload_workspace called with id: {:?}", id);
    let mut workspaces = open_workspaces().write();
    workspaces.remove(id.id);
    tracing::info!("Workspace unloaded with id: {:?}", id);
}

/// Permanently delete a workspace from the filesystem.
#[tauri::command]
pub fn delete_workspace(id: WorkspaceId) {
    tracing::info!("delete_workspace called with id: {:?}", id);
    let workspace = get_workspace_ref(id);
    let path = workspace.location.clone();
    _ = std::fs::remove_dir_all(path);
    tracing::info!("Workspace deleted with id: {:?}", id);
}

/// Remove a note from a specific path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
pub async fn files_in_workspace(workspace_id: WorkspaceId) -> Vec<ContextualDocument> {
    tracing::info!("files_in_workspace called with id: {:?}", workspace_id);
    let workspace = get_workspace_ref(workspace_id);
    workspace.files().await.unwrap()
}
