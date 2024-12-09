/*!
# Prologue Comments
## Name of Code Artifact: Note Management and Search Application Framework

## Brief Description: This code provides a framework for managing notes, including functionality to save, read, delete, and tag notes within workspaces. It also incorporates contextual search capabilities using a BERT-based language model and integrates with Tauri for application development.

## Programmer’s Name: Evan Almoff, Suhaan

## Date Created: 2024-10-14
## Dates Revised and Description of Revisions: 
## -> 2024-10-15: Added search functionality and workspace management. 
## -> 2024-10-16: Refactored code for better modularity and testability.
## -> 2024-10-17: Added error handling and documentation.
## -> 2024-10-18: Integrated with Tauri for application development.
## -> 2024-12-08: Improved performance and fixed bugs.
## -> 2024-12-08: Finalized documentation and testing.

## Preconditions: A Tauri application context is required for the run function.
The BERT model must be initialized using the bert function before using search functionalities dependent on embeddings.
Workspaces must be loaded for note-related operations.
Acceptable and Unacceptable Input Values/Types:

### Note titles and contents must be strings.
### Workspace paths must be valid directory paths.
### Search queries must be strings.
### Tags should conform to the structure defined in the code.
### Invalid workspace paths, malformed data, or missing inputs can cause errors.

## Postconditions:

### Notes are saved, retrieved, or deleted in the workspace context.
### Searches return relevant results based on embeddings or fail with descriptive errors.
### Workspace integrity is maintained after all operations.

## Return Values/Types:

### Functions return Result types with success yielding appropriate outputs (e.g., search results, note metadata).
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
use note::{get_tags, read_note, remove_note, save_note, set_tags, ContextualDocument};
use search::{context_search, search};
/// The line `use std::{ num::NonZero, sync::{Arc, OnceLock} };` is importing specific items from the
/// `std` (standard library) module in Rust. Here's what each item does:
use std::{
    num::NonZero, // NonZero is a type that represents a non-zero integer. It's used in the `bert` function to set the cache size.
    sync::{Arc, OnceLock}, // Arc is a type that provides shared ownership of a value. OnceLock is a type that ensures a value is only initialized once.
};
use surrealdb::engine::local::Db; // surrealdb is a database engine that provides local storage capabilities.
/// The line `use workspace::{ delete_workspace, files_in_workspace, get_workspace_id, load_workspace,
/// unload_workspace };` is importing specific functions and identifiers from the `workspace` module in
/// Rust. Here's what each of these imported items does:
use workspace::{ 
    delete_workspace, files_in_workspace, get_workspace_id, load_workspace, unload_workspace, // These functions are used to manage workspaces.
};

#[cfg(test)]
use pretty_assertions::assert_eq;

mod classifier;
mod note;
mod search;
mod workspace;

/// The line `static BERT: OnceLock<anyhow::Result<Arc<CachedEmbeddingModel<Bert>>>> = OnceLock::new();`
/// is declaring a static variable named `BERT` of type `OnceLock` that holds a result of type
/// `anyhow::Result` containing an `Arc` (atomic reference counting) to a `CachedEmbeddingModel`
/// specialized with `Bert`.
static BERT: OnceLock<anyhow::Result<Arc<CachedEmbeddingModel<Bert>>>> = OnceLock::new();
/// The line `static BERT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());` is
/// declaring a static variable named `BERT_LOCK` of type `tokio::sync::Mutex<()>`.
static BERT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The function `bert` is an asynchronous Rust function that retrieves a cached embedding model of
/// Bert, ensuring only one instance is created using a lock.
/// 
/// Returns:
/// 
/// The function `bert()` returns a `Result` containing a reference to a static `Arc` pointing to a
/// `CachedEmbeddingModel` of type `Bert`.
async fn bert() -> anyhow::Result<&'static Arc<CachedEmbeddingModel<Bert>>> {
    let _guard = BERT_LOCK.lock().await; // Acquire a lock on the BERT_LOCK mutex.
    if BERT.get().is_none() { // Check if the BERT static variable is uninitialized.
        _ = BERT.set( // Set the BERT static variable to a new value.
            Bert::builder() // Create a new Bert model builder.
                .with_source(BertSource::snowflake_arctic_embed_small()) // Set the source of the Bert model.
                .build() // Build the Bert model. 
                .await // Build the Bert model asynchronously.
                .map(|e| Arc::new(e.cached(NonZero::new(2048).unwrap()))), // Cache the Bert model with a size of 2048.
        );
    }
    BERT.get() // Return the value of the BERT static variable.
        .unwrap() // Unwrap the value of the BERT static variable.
        .as_ref() // Get a reference to the value of the BERT static variable.
        .map_err(|err| anyhow::anyhow!(err)) // Map any error to an anyhow error.
}

/// The line `type ContextualDocumentTable = DocumentTable<Db, ContextualDocument,
/// Arc<CachedEmbeddingModel<Bert>>, DefaultSentenceChunker>;` is defining a type alias in Rust.
type ContextualDocumentTable =
    DocumentTable<Db, ContextualDocument, Arc<CachedEmbeddingModel<Bert>>, DefaultSentenceChunker>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// The function initializes a Tauri application with various plugins and handlers for workspace and
/// note management.
pub fn run() {
    tauri::Builder::default() // Create a new Tauri application builder with default settings.
        .plugin(tauri_plugin_shell::init()) // Initialize the Tauri shell plugin.
        .plugin(tauri_plugin_fs::init()) // Initialize the Tauri file system plugin.
        .invoke_handler(tauri::generate_handler![ // Generate an invoke handler for the following functions.
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

/// The test function `test_notes` in Rust performs various operations related to saving, removing,
/// searching, and asserting notes within a workspace.
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
