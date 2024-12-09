/*!
# Prologue Comments
## Name of Code Artifact: Search Application Framework

## Brief Description: This code provides a framework for searching notes, including functionality to search for notes based on text and tags, as well as to search for context around a cursor position in a note. It integrates with Tauri for application development.
## Programmer’s Name: Evan Almoff

## Date Created: 2024-10-14
## Dates Revised and Description of Revisions: 
## -> 2024-10-15: Added search functionality and workspace management.
## -> 2024-10-16: Improved search functionality and added error handling.
## -> 2024-10-17: Fixed bugs and improved performance.
## -> 2024-10-18: Added support for multiple workspaces.
## -> 2024-11-14: Added support for context searching
## -> 2024-12-08: Finalized documentation and testing.


## Preconditions: A Tauri application context is required for the run function.
The BERT model must be initialized using the bert function before using search functionalities dependent on embeddings.
Workspaces must be loaded for note-related operations.
Acceptable and Unacceptable Input Values/Types:

## Postconditions:

### Searches return relevant results based on embeddings or fail with descriptive errors.

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

use kalosm::{language::*, IntoEmbeddingIndexedTableSearchFilter}; // Import the necessary modules from the kalosm crate.
use serde::{Deserialize, Serialize}; // Import the necessary modules from the serde crate.
use std::ops::Range; // Import the Range module from the standard library.
use surrealdb::sql::Id; // Import the Id module from the surrealdb crate.

use crate::bert;    // Import the bert module from the current crate.
use crate::classifier::chunk_text;
use crate::workspace::{get_workspace_ref, WorkspaceId};

#[derive(Serialize, Deserialize)]
struct MetaId {
    id: String, // The id of the document
} 

#[derive(Serialize, Deserialize)]
/// Represents the result of a search operation.
///
/// # Fields
///
/// * `distance` - A floating-point value representing the distance or relevance of the search result.
/// * `title` - A string containing the title of the search result.
/// * `character_range` - A range of character indices indicating the position of the search result within the source text.
pub struct SearchResult {
    pub distance: f32,
    pub title: String,
    pub character_range: Range<usize>,
}

#[tauri::command]
pub async fn search(
    text: String,
    tags: Vec<String>,
    results: usize,
    workspace_id: WorkspaceId,
) -> Result<Vec<SearchResult>, String> {
    tracing::info!("Search called with text {:?} and tags {:?}", text, tags);
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace
        .document_table()
        .await
        .map_err(|e| e.to_string())?;
    let bert = bert().await.map_err(|e| e.to_string())?;
    let embedding = bert
        .embed_for(EmbeddingInput::new(text, EmbeddingVariant::Query))
        .await
        .map_err(|e| e.to_string())?;
    /// Queries the database for documents that contain all specified tags.
    ///
    /// This function constructs and executes a SQL query to retrieve the IDs of documents
    /// from the `document_table` that contain all the tags specified in the `tags` vector.
    /// The query uses the `CONTAINSALL` function to ensure that all tags are present in the
    /// document's tags.
    ///
    /// # Arguments
    ///
    /// * `document_table` - A reference to the table containing the documents.
    /// * `tags` - A vector of tags that the documents must contain.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of document IDs if the query is successful, or an error
    /// message as a `String` if the query fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the query execution fails or if there is an issue
    /// with serializing the `tags` vector to a JSON string.
    let mut documents_with_all_tags = document_table
        .table()
        .db()
        .query(format!(
            "SELECT meta::id(id) as id FROM {} WHERE tags.name CONTAINSALL {}",
            document_table.table().table(),
            serde_json::to_string(&tags).unwrap()
        ))
        .await
        .map_err(|e| e.to_string())?;

    let documents_with_all_tags: Vec<MetaId> =
        documents_with_all_tags.take(0).map_err(|e| e.to_string())?;
    let nearest = document_table
        .search(embedding)
        .with_results(results)
        .with_filter(
            documents_with_all_tags
                .into_iter()
                .map(|id| Id::String(id.id)),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(nearest
        .into_iter()
        .map(|result| {
            let title = result.record.document.title().to_string();
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
                title,
                character_range,
            }
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct ContextResult {
    /// The distance from the search result to the cursor
    pub distance: f32,
    /// The title of the document
    pub title: String,
    /// The text of the search result. This includes the context sentences around the search result
    pub text: String,
    /// The utf16 index of the most relevant section of the search result within [`ContextResult::text`]
    pub relevant_range: Range<usize>,
}

// Take a list of sentence ranges and return the range of sentences of a specific length around the target sentence
fn get_sentence_range( // Function to get the range of sentences of a specific length around the target sentence.
    sentences: &[Range<usize>], // A slice of sentence ranges.
    target_sentence_index: usize,
    sentences_to_include: usize,
) -> Range<usize> {
    let sentence_embedding_range_end =
        (target_sentence_index + sentences_to_include / 2).min(sentences.len()); // The end of the sentence embedding range.
    let sentence_embedding_range_start = 
        sentence_embedding_range_end.saturating_sub(sentences_to_include); // The start of the sentence embedding range.
    sentence_embedding_range_start..sentence_embedding_range_end // Return the range of sentences of a specific length around the target sentence.
}

fn utf8_range_to_utf16_range(utf8_range: Range<usize>, text: &str) -> Range<usize> {
    let utf16_start = text[..utf8_range.start]
        .chars() // Convert the text to a character iterator.
        .map(|c| c.len_utf16()) // Map each character to its utf16 length.
        .sum(); // Sum the utf16 lengths of all characters before the range.
    let utf16_len: usize = text[utf8_range.start..utf8_range.end] // Get the utf16 length of the range.
        .chars()
        .map(|c| c.len_utf16()) // Map each character to its utf16 length.
        .sum();
    utf16_start..utf16_start + utf16_len // Return the utf16 range.
}

#[tauri::command]
pub async fn context_search(
    // The title of the document the cursor is in
    document_title: Option<String>,
    // The entire text of the document we are generating context for
    document_text: String,
    // The character index of the cursor in utf16 bytes
    cursor_utf16_index: usize,
    // The number of results to return
    results: usize,
    // The number of sentences of context to return around the search result
    context_sentences: usize,
    // The workspace to search in
    workspace_id: WorkspaceId,
) -> Result<Vec<ContextResult>, String> {
    tracing::info!("Search called with title {:?}, text {:?}, character index {:?}, results {:?}, and context_sentences {:?}", document_title, document_text, cursor_utf16_index, results, context_sentences);
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace
        .document_table()
        .await
        .map_err(|err| format!("{}", err))?;
    // First split up the text into sentences
    let sentences = chunk_text(&document_text);
    tracing::info!(
        "Split into sentences {:?}",
        sentences
            .iter()
            .map(|range| &document_text[range.clone()])
            .collect::<Vec<_>>()
    );
    let mut cursor_byte_index = None;
    let mut current_utf16_index = 0;
    for (byte_index, char) in document_text.char_indices() {
        current_utf16_index += char.len_utf16();
        if current_utf16_index >= cursor_utf16_index {
            cursor_byte_index = Some(byte_index);
        }
    }
    let cursor_byte_index = cursor_byte_index
        .ok_or_else(|| "Cannot search around a sentence that is not in the document".to_string())?;
    tracing::info!("Cursor byte index: {:?}", cursor_byte_index);
    let cursor_sentence_index = sentences
        .iter()
        .position(|range| cursor_byte_index <= range.end)
        .unwrap_or(sentences.len() - 1);
    tracing::info!("Cursor sentence index: {:?}", cursor_sentence_index);
    // Find 3 sentences around the cursor sentence
    let sentence_embedding_range = get_sentence_range(&sentences, cursor_sentence_index, 3);
    tracing::info!("Sentence embedding range: {:?}", sentence_embedding_range);
    let context = sentences[sentence_embedding_range]
        .into_iter()
        .map(|range| &document_text[range.clone()])
        .collect::<String>();
    tracing::info!("Searching with context {:?}", context);

    // Embed the context
    let bert = bert().await.map_err(|err| format!("{}", err))?;
    let embedding = bert
        .embed(context)
        .await
        .map_err(|err| format!("{}", err))?;

    // And search for the nearest results
    let mut search = document_table.search(embedding).with_results(results);

    // Filter out the current document from the search results if it has been saved
    if let Some(document_title) = document_title {
        // Get the ids of all documents with a different title
        let mut all_other_document_ids = document_table
            .table()
            .db()
            .query(format!(
                "SELECT meta::id(id) as id FROM {} WHERE document.title != \"{}\"",
                document_table.table().table(),
                document_title
            ))
            .await
            .map_err(|e| e.to_string())?;

        let all_other_document_ids: Vec<MetaId> =
            all_other_document_ids.take(0).map_err(|e| e.to_string())?;

        // Only include those documents in the search results
        search = search.with_filter(
            all_other_document_ids
                .into_iter()
                .map(|id| Id::String(id.id))
                .into_embedding_indexed_table_search_filter(&document_table.table())
                .await
                .map_err(|e| e.to_string())?,
        );
    }

    let nearest = search.await.map_err(|err| format!("{}", err))?;

    tracing::info!("Nearest results: {:?}", nearest);

    Ok(nearest
        .into_iter()
        .map(|result| {
            let title = result.record.document.title().to_string();
            let body = result.record.document.body();

            let result_chunks = chunk_text(&body);

            let target_sentence = result_chunks
                .iter()
                .position(|range| range.contains(&result.byte_range.start))
                .unwrap_or(result_chunks.len() - 1);
            let target_sentence_utf8_range = result_chunks[target_sentence].clone();

            let context_sentence_range =
                get_sentence_range(&result_chunks, target_sentence, context_sentences);

            let context_utf8_range = result_chunks[context_sentence_range.start].start
                ..result_chunks[context_sentence_range.end - 1].end;

            let text = body[context_utf8_range.clone()].to_string();
            let distance = result.distance;
            let relevant_range = utf8_range_to_utf16_range(
                (target_sentence_utf8_range.start - context_utf8_range.start)
                    ..(target_sentence_utf8_range.end - context_utf8_range.start),
                &text,
            );
            tracing::info!(
                "Results: distance {:?} title {:?} relevant_range {:?} text {:?}",
                distance,
                title,
                relevant_range,
                text
            );
            ContextResult {
                distance,
                title,
                relevant_range,
                text,
            }
        })
        .collect())
}

#[tokio::test]
async fn test_note_context() {
    use crate::{delete_workspace, load_workspace, remove_note, save_note, unload_workspace};
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
        "The math is mathing QED. The math is mathing QED. This is my note. The cat is here. Yes it is.".to_string(),
        workspace,
    )
    .await.unwarp();
    let results =
        crate::search::context_search(None, "The cat is here".to_string(), 0, 1, 3, workspace)
            .await
            .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "search-note");
    assert_eq!(
        results[0].text,
        "The math is mathing QED. This is my note. The cat is here. Yes it is."
    );
    assert_eq!(results[0].relevant_range, 42..59);

    delete_workspace(workspace);
    unload_workspace(workspace);
}
