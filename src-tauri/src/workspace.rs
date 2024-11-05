use kalosm::language::*;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use serde::{Deserialize, Serialize};
use slab::Slab;
use std::path::PathBuf;
use std::sync::OnceLock;
use surrealdb::{engine::local::RocksDb, Surreal};

use crate::note::ContextualDocument;
use crate::{bert, ContextualDocumentTable};

pub struct Workspace {
    pub location: PathBuf,
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

    pub fn document_path(&self, title: &str) -> anyhow::Result<PathBuf> {
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

    async fn files(&self) -> anyhow::Result<Vec<ContextualDocument>> {
        #[derive(Serialize, Deserialize)]
        struct FilePath {
            path: PathBuf,
        }

        let table = self.document_table().await?;
        let paths: Vec<ContextualDocument> =
            table.table().db().select(table.table().table()).await?;
        Ok(paths)
    }

    pub async fn document_table(&self) -> anyhow::Result<&ContextualDocumentTable> {
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
    tracing::info!("open_workspaces called");
    OPEN_WORKSPACES.get_or_init(|| RwLock::new(Slab::new()))
}

/// Get a reference to a workspace by the id
pub fn get_workspace_ref(id: WorkspaceId) -> MappedRwLockReadGuard<'static, Workspace> {
    tracing::info!("get_workspace_ref called with id: {:?}", id);
    RwLockReadGuard::map(open_workspaces().read(), |slab| slab.get(id.id).unwrap())
}

/// Load a workspace at a path into memory. This will either load the existing workspace from the filesystem or create a new workspace at the path.
#[tauri::command]
pub fn load_workspace(path: PathBuf) -> WorkspaceId {
    tracing::info!("Loading workspace at {:?}", path);
    let mut workspaces = open_workspaces().write();
    let new_workspace = Workspace::new(path);
    let id = workspaces.insert(new_workspace);
    tracing::info!("Workspace loaded with id: {:?}", id);
    WorkspaceId { id }
}

#[tauri::command]
pub fn get_workspace_id(path: PathBuf) -> WorkspaceId {
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

/// Unload a workspace from memory. This should be called whenever the workspace is closed.
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
