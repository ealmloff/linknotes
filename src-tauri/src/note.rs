use kalosm::language::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use surrealdb::sql::Id;

use crate::bert;
use crate::workspace::{get_workspace_ref, WorkspaceId};

#[derive(Serialize, Deserialize)]
struct ContextualDocumentLocation {
    document_id: Id,
    pub location: PathBuf,
    segments: Vec<Segment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextualDocument {
    pub document: Document,
    pub tags: Vec<Tag>,
}

impl AsRef<Document> for ContextualDocument {
    fn as_ref(&self) -> &Document {
        &self.document
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    name: String,
    manual: bool,
}

const DOCUMENT_NAME_TABLE: &str = "document_paths";

#[derive(Serialize, Deserialize, PartialEq)]
struct Segment {
    source_char_range: Range<usize>,
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
#[error("document does not exist")]
pub struct DocumentDoesNotExistError;

/// Set the tags for a note with the given a title in the workspace. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
pub async fn set_tags(
    title: String,
    tags: Vec<Tag>,
    workspace_id: WorkspaceId,
) -> Result<(), DocumentDoesNotExistError> {
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();
    let table_name = document_table.table().table();
    let location: ContextualDocumentLocation = db
        .select((DOCUMENT_NAME_TABLE, title))
        .await
        .unwrap()
        .ok_or(DocumentDoesNotExistError)?;
    let id = location.document_id;
    db.query(format!(
        "UPDATE {}:{} SET tags = {}",
        table_name,
        id,
        serde_json::to_string(&tags).unwrap()
    ))
    .await
    .unwrap();

    Ok(())
}

/// Save a note with a title, and contents in a workspace. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
pub async fn add_note(title: String, text: String, workspace_id: WorkspaceId) {
    tracing::info!("Add_note called");
    tracing::info!("Workspace added with id: {:?}", workspace_id);

    let workspace = get_workspace_ref(workspace_id);
    let document_path = workspace.document_path(&title).unwrap();

    // Write the note content to the file
    fs::write(&document_path, &text).unwrap();

    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();
    let document = Document::from_parts(title.clone(), text);
    tracing::info!("Chunking document");
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
    tracing::info!("Looking for existing document");
    // First check if the document already exists
    let current_location: Option<ContextualDocumentLocation> = db
        .select((DOCUMENT_NAME_TABLE, title.as_str()))
        .await
        .unwrap();
    let mut tags = Vec::new();
    if let Some(current_location) = &current_location {
        let previous_document = document_table
            .select(current_location.document_id.clone())
            .await
            .unwrap();
        // If this is the same as the note already in the db, just return
        if current_location.segments == segments && previous_document.document == document {
            return;
        }
        tags = previous_document.tags.clone();
        // Delete the old document if it exists
        document_table
            .delete(current_location.document_id.clone())
            .await
            .unwrap();
    }

    let contextual = ContextualDocument { document, tags };
    tracing::info!("Inserting document with id: {:?}", contextual);
    let document_id = document_table
        .insert_with_chunks(contextual, chunks)
        .await
        .unwrap();
    tracing::info!("Document inserted successfully");

    let location = ContextualDocumentLocation {
        document_id,
        location: document_path,
        segments,
    };

    // If it doesn't, create it
    if current_location.is_none() {
        let _: Option<ContextualDocumentLocation> = db
            .create((DOCUMENT_NAME_TABLE, title.as_str()))
            .content(location)
            .await
            .unwrap();
    }
    // Otherwise, update it
    else {
        let _: Option<ContextualDocumentLocation> = db
            .update((DOCUMENT_NAME_TABLE, title.as_str()))
            .content(location)
            .await
            .unwrap();
    }
}

/// Remove a note from a specific path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
pub async fn remove_note(title: String, workspace_id: WorkspaceId) {
    tracing::info!("Removing note with title: {:?}", title);
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();
    // First check if the document already exists
    let current_location: Option<ContextualDocumentLocation> =
        db.delete((DOCUMENT_NAME_TABLE, &title)).await.unwrap();
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
pub async fn read_file(
    title: String,
    workspace_id: WorkspaceId,
) -> Result<String, DocumentDoesNotExistError> {
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();
    let location: ContextualDocumentLocation = db
        .select((DOCUMENT_NAME_TABLE, &title))
        .await
        .unwrap()
        .ok_or(DocumentDoesNotExistError)?;
    Ok(std::fs::read_to_string(location.location).unwrap())
}

#[tokio::test]
async fn test_set_tags() {
    use crate::workspace::{
        delete_workspace, files_in_workspace, load_workspace, unload_workspace,
    };
    use std::env::temp_dir;

    _ = tracing_subscriber::fmt::try_init();

    let temp = temp_dir();
    let workspace = load_workspace(temp);
    let title = "test-note".to_string();
    let text = "test note".to_string();
    add_note(title.clone(), text.clone(), workspace).await;
    let tags = vec![Tag {
        name: "test".to_string(),
        manual: true,
    }];
    set_tags(title.clone(), tags.clone(), workspace)
        .await
        .unwrap();
    let notes = files_in_workspace(workspace).await;

    assert_eq!(
        notes,
        vec![ContextualDocument {
            document: Document::from_parts(title, text),
            tags
        }]
    );
    delete_workspace(workspace);
    unload_workspace(workspace);
}
