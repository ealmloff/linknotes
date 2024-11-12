use kalosm::language::*;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use surrealdb::sql::Id;

use crate::bert;
use crate::workspace::{get_workspace_ref, WorkspaceId};

#[derive(Serialize, Deserialize)]
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
    println!("Search called with text {:?} and tags {:?}", text, tags);
    tracing::info!("Search called with text {:?} and tags {:?}", text, tags);
    let workspace = get_workspace_ref(workspace_id);
    let document_table = workspace.document_table().await.map_err(|e| e.to_string())?;
    let bert = bert().await.map_err(|e| e.to_string())?;
    let embedding = bert.embed(text).await.map_err(|e| e.to_string())?;
    let mut documents_with_all_tags = document_table
        .table()
        .db()
        .query(dbg!(format!(
            "SELECT meta::id(id) as id FROM {} WHERE tags.name CONTAINSALL {}",
            document_table.table().table(),
            serde_json::to_string(&tags).unwrap()
        )))
        .await
        .map_err(|e| e.to_string())?;

    #[derive(Serialize, Deserialize)]
    struct MetaId {
        id: String,
    }

    let documents_with_all_tags: Vec<MetaId> = documents_with_all_tags.take(0).map_err(|e| e.to_string())?;
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