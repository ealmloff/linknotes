use kalosm::language::*;
use serde::{Deserialize, Serialize};
use std::ops::Range;

use crate::bert;
use crate::workspace::{get_workspace_ref, WorkspaceId};

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub distance: f32,
    pub title: String,
    pub character_range: Range<usize>,
}

/// Search for some text in the notes. Returns a list of results with the distance, path and character range of each result.
#[tauri::command]
pub async fn search(text: String, results: usize, workspace_id: WorkspaceId) -> Vec<SearchResult> {
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
        .collect()
}
