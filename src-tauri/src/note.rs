use kalosm::language::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use surrealdb::sql::Id;

#[cfg(test)]
use pretty_assertions::assert_eq;

use crate::bert;
use crate::classifier::chunk_text;
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord)]
pub struct Tag {
    pub name: String,
    pub manual: bool,
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
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
    mut tags: Vec<Tag>,
    workspace_id: WorkspaceId,
) -> Result<(), DocumentDoesNotExistError> {
    tracing::info!("set_tags called with title {:?} and tags {:?}", title, tags);
    let workspace = get_workspace_ref(workspace_id);
    workspace.retrain_classifier();
    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();
    let table_name = document_table.table().table();
    let location: ContextualDocumentLocation = db
        .select((DOCUMENT_NAME_TABLE, title))
        .await
        .unwrap()
        .ok_or(DocumentDoesNotExistError)?;
    let note: ContextualDocument = document_table
        .select(location.document_id.clone())
        .await
        .unwrap();
    let automatic_tags = note.tags.into_iter().filter(|tag| !tag.manual);
    tags.extend(automatic_tags);
    tags.sort();
    tags.dedup();
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

#[tauri::command]
pub async fn get_tags(
    title: String,
    workspace_id: WorkspaceId,
) -> Result<Vec<Tag>, DocumentDoesNotExistError> {
    tracing::info!("get_tags called with title {:?}", title);
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();
    let location: ContextualDocumentLocation = db
        .select((DOCUMENT_NAME_TABLE, &title))
        .await
        .unwrap()
        .ok_or(DocumentDoesNotExistError)?;
    let note: ContextualDocument = document_table.select(location.document_id).await.unwrap();
    Ok(note.tags)
}

/// Save a note with a title, and contents in a workspace. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
pub async fn save_note(
    title: String,
    text: String,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
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
    let body = document.body();
    let sentences = chunk_text(body);
    let bert = bert().await.unwrap();
    let embeddings = bert
        .embed_batch(sentences.iter().map(|sentence| &body[sentence.clone()]))
        .await
        .unwrap();
    let chunks = sentences
        .clone()
        .into_iter()
        .zip(embeddings.into_iter())
        .map(|(byte_range, embedding)| Chunk {
            byte_range,
            embeddings: vec![embedding],
        });
    let segments = sentences
        .iter()
        .map(|byte_range| Segment {
            source_char_range: byte_range.clone(),
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
            return Ok(());
        }
        tags = previous_document.tags.clone();
        // Delete the old document if it exists
        document_table
            .delete(current_location.document_id.clone())
            .await
            .unwrap();
    }

    tags.retain(|tag| tag.manual);
    // Classify the document and add an automatic tag
    let mut contextual = ContextualDocument { document, tags };
    let tag = workspace
        .classify(&contextual)
        .await
        .map_err(|err| err.to_string())?;
    contextual.tags.push(tag);

    tracing::info!("Inserting document with id: {:?}", contextual);
    let document_id = document_table
        .insert_with_chunks(contextual, chunks)
        .await
        .map_err(|err| err.to_string())?;
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
            .map_err(|err| err.to_string())?;
    }
    // Otherwise, update it
    else {
        let _: Option<ContextualDocumentLocation> = db
            .update((DOCUMENT_NAME_TABLE, title.as_str()))
            .content(location)
            .await
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}

/// Remove a note from a specific path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
pub async fn remove_note(title: String, workspace_id: WorkspaceId) {
    tracing::info!("Removing note with title: {:?}", title);
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();

    // Get the document path
    let document_path = workspace.document_path(&title).unwrap();

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

    // Remove the .txt file
    if document_path.exists() {
        fs::remove_file(document_path).unwrap();
    }
}

/// Remove a note from a specific path. The path should be canonicalized so it is consistent regardless of the working directory.
#[tauri::command]
pub async fn read_note(
    title: String,
    workspace_id: WorkspaceId,
) -> Result<ContextualDocument, DocumentDoesNotExistError> {
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.unwrap();
    let db = document_table.table().db();
    let location: ContextualDocumentLocation = db
        .select((DOCUMENT_NAME_TABLE, &title))
        .await
        .unwrap()
        .ok_or(DocumentDoesNotExistError)?;
    let note: ContextualDocument = document_table.select(location.document_id).await.unwrap();
    Ok(note)
}

#[tokio::test]
async fn test_set_tags() {
    use crate::note::save_note;
    use crate::workspace::{
        delete_workspace, files_in_workspace, load_workspace, unload_workspace,
    };
    use std::env::temp_dir;

    _ = tracing_subscriber::fmt::try_init();

    let temp = temp_dir();
    let workspace = load_workspace(temp);
    let title = "test-note".to_string();
    let text = "test note".to_string();
    save_note(title.clone(), text.clone(), workspace)
        .await
        .unwrap();
    let tags = vec![
        Tag {
            name: "tag1".to_string(),
            manual: true,
        },
        Tag {
            name: "tag2".to_string(),
            manual: true,
        },
    ];
    set_tags(title.clone(), tags.clone(), workspace)
        .await
        .unwrap();

    let title2 = "my-other-test-note".to_string();
    let text2 = "testing other note".to_string();
    save_note(title2.clone(), text2.clone(), workspace)
        .await
        .unwrap();
    let tags2 = vec![
        Tag {
            name: "tag2".to_string(),
            manual: true,
        },
        Tag {
            name: "tag3".to_string(),
            manual: true,
        },
    ];
    set_tags(title2.clone(), tags2.clone(), workspace)
        .await
        .unwrap();

    let notes = files_in_workspace(workspace).await;

    assert_eq!(
        notes,
        vec![
            ContextualDocument {
                document: Document::from_parts(title, text.clone()),
                tags
            },
            ContextualDocument {
                document: Document::from_parts(title2, text2.clone()),
                tags: tags2
            }
        ]
    );

    let results =
        crate::search::search("test".to_string(), vec!["tag2".to_string()], 10, workspace)
            .await
            .unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "test-note");
    assert_eq!(results[0].character_range, 0..text.len());
    assert_eq!(results[1].title, "my-other-test-note");
    assert_eq!(results[1].character_range, 0..text2.len());

    let results = crate::search::search(
        "test".to_string(),
        vec!["tag1".to_string(), "tag2".to_string()],
        10,
        workspace,
    )
    .await
    .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "test-note");
    assert_eq!(results[0].character_range, 0..text.len());

    let results = crate::search::search(
        "my note is here".to_string(),
        vec!["testing".to_string()],
        10,
        workspace,
    )
    .await
    .unwrap();
    assert!(results.is_empty());

    delete_workspace(workspace);
    unload_workspace(workspace);
}
