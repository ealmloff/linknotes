use kalosm::language::*;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
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

use crate::bert;
use crate::workspace::{get_workspace_ref, WorkspaceId};

#[derive(Serialize, Deserialize)]
struct ContextualDocumentLocation {
    document_id: Id,
    pub location: PathBuf,
    segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize)]
pub struct ContextualDocument {
    pub path: String,
    pub document: Document,
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
pub async fn add_note(title: String, text: String, workspace_id: WorkspaceId) {
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

/// Remove a note from a specific path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
pub async fn remove_note(path: PathBuf, workspace_id: WorkspaceId) {
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
pub async fn read_file(path: PathBuf) -> String {
    std::fs::read_to_string(path).unwrap()
}
