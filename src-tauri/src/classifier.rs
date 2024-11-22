use kalosm::language::{
    accelerated_device_if_available, Chunker, DefaultSentenceChunker, Document, SentenceChunker,
};
use kalosm_learning::{
    Classifier, ClassifierConfig, ClassifierProgress, TextClassifierDatasetBuilder,
};
use std::ops::Range;

use crate::{
    bert,
    note::{ContextualDocument, Tag},
    workspace::Workspace,
};

pub(crate) fn chunk_text(text: &str) -> Vec<Range<usize>> {
    // First split based on bullet points
    let mut segments = Vec::new();
    let mut last_idx = 0;
    let mut iter = text.char_indices();
    let mut add_line = |text: &str, range: Range<usize>, last_idx: &mut usize| {
        let line = &text[range.clone()];
        let mut line_chars = line.char_indices().skip_while(|(_, c)| c.is_whitespace());
        match line_chars.next() {
            Some((idx, '-')) => {
                let mut idx = idx;
                while let Some((new_idx, c)) = line_chars.next() {
                    idx = new_idx;
                    if !c.is_whitespace() {
                        break;
                    }
                }
                let range = range.start + idx..range.end;
                *last_idx = range.end;
                segments.push(range);
            }
            Some((idx, c)) if c.is_numeric() => {
                let mut idx = idx;
                while let Some((new_idx, c)) = line_chars.next() {
                    idx = new_idx;
                    if !(c.is_numeric() || c == ')' || c == '.' || c.is_whitespace()) {
                        break;
                    }
                }
                let range = range.start + idx..range.end;
                *last_idx = range.end;
                segments.push(range);
            }
            _ => {}
        }
    };
    loop {
        match iter.next() {
            Some((idx, '\n')) => {
                add_line(&text, last_idx..idx, &mut last_idx);
            }
            Some((_, _)) => continue,
            None => break,
        }
    }
    add_line(&text, last_idx..text.len(), &mut last_idx);
    if last_idx < text.len() {
        let remaining = &text[last_idx..];
        let mut idx = 0;
        let mut iter = remaining.char_indices();
        while let Some((new_idx, c)) = iter.next() {
            idx = new_idx;
            if !c.is_whitespace() {
                break;
            }
        }
        let mut back_idx = remaining.len();
        let mut iter = remaining.char_indices().rev();
        while let Some((new_idx, c)) = iter.next() {
            if !c.is_whitespace() {
                break;
            }
            back_idx = new_idx;
        }
        segments.push(last_idx + idx..last_idx + back_idx);
    }
    let mut ranges = Vec::new();
    for segment_range in segments {
        ranges.extend(
            SentenceChunker::default()
                .split_sentences(&text[segment_range.clone()])
                .into_iter()
                .map(|range| {
                    let start = segment_range.start + range.start;
                    let end = segment_range.start + range.end;
                    start..end
                }),
        );
    }
    ranges
}

#[test]
fn test_chunk_text() {
    let text = "- this is a note\n    \t- this is a nested note\n- this is another note. This is another\n\r\t12. hello world\n\t3) this is a numbered list\n- this is another note\nThis\nis\na\nnote\n   ";
    let ranges = chunk_text(text);
    assert_eq!(
        ranges
            .into_iter()
            .map(|range| &text[range])
            .collect::<Vec<_>>(),
        vec![
            "this is a note",
            "this is a nested note",
            "this is another note. ",
            "This is another",
            "hello world",
            "this is a numbered list",
            "this is another note",
            "This\nis\na\nnote",
        ]
    )
}

pub struct TagClassifier {
    classifier: Classifier<u32>,
}

impl TagClassifier {
    pub async fn new(
        workspace: &Workspace,
        documents: &[ContextualDocument],
        progress: impl Fn(ClassifierProgress),
    ) -> Self {
        let device = accelerated_device_if_available().unwrap();
        let mut tagged_documents = Vec::new();
        let default_docs = default_documents();
        for document in documents.iter().chain(default_docs.iter()) {
            let text = document.document.body();
            let chunks = chunk_text(text);
            for tag in &document.tags {
                let id = { workspace.get_tag_id(tag.name.as_str()) };
                for chunk in &chunks {
                    tagged_documents.push((&text[chunk.clone()], id));
                }
            }
        }
        let class_count = { workspace.tag_count() as u32 };
        let config = ClassifierConfig::new().classes(class_count);
        let classifier = Classifier::new(&device, config).unwrap();

        let device = accelerated_device_if_available().unwrap();
        let epochs = 5;
        let learning_rate = 0.003;
        let batch_size = 50;
        let bert = bert().await.unwrap();
        let mut dataset = TextClassifierDatasetBuilder::<u32, _>::new(bert);
        for (document, id) in tagged_documents {
            dataset.add(document, id).await.unwrap();
        }
        let dataset = dataset.build(&device).unwrap();
        classifier
            .train(&dataset, epochs, learning_rate, batch_size, progress)
            .unwrap();

        Self { classifier }
    }

    pub async fn classify(
        &self,
        workspace: &Workspace,
        document: &ContextualDocument,
    ) -> anyhow::Result<Tag> {
        let bert = bert().await?;
        let chunks = DefaultSentenceChunker
            .chunk(&document.document, bert)
            .await?;
        let classes = chunks
            .into_iter()
            .flat_map(|chunk| chunk.embeddings)
            .map(|embedding| self.classifier.run(&embedding.to_vec()))
            .fold(
                Ok(None),
                |current: anyhow::Result<Option<Vec<f32>>>, new| {
                    let new = new?;
                    let mut new = new.classes().to_vec();
                    new.sort_by_key(|class| class.0);
                    let new = new.into_iter().map(|class| class.1).collect::<Vec<_>>();
                    Ok(match current? {
                        None => Some(new),
                        Some(current) => {
                            Some(current.iter().zip(new).map(|(a, b)| a + b).collect())
                        }
                    })
                },
            )?;
        let Some(classes) = classes else {
            return Err(anyhow::anyhow!("Empty input for classification"));
        };
        let most_likely_tag = classes
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0 as u32;
        let tag_id = workspace.get_tag_name(most_likely_tag);
        Ok(Tag {
            name: tag_id.to_string(),
            manual: false,
        })
    }
}

const INTEGRALS_NOTE: &str = include_str!("./classifier-notes/integrals.note");
const SIMD_NOTE: &str = include_str!("./classifier-notes/simd.note");
const DISCRETE_MATH_NOTE: &str = include_str!("./classifier-notes/discrete-math.note");
const REACTIVITY_NOTE: &str = include_str!("./classifier-notes/reactivity.note");
const OS_NOTE: &str = include_str!("./classifier-notes/os.note");
const STATS_NOTE: &str = include_str!("./classifier-notes/stats.note");

fn default_documents() -> Vec<ContextualDocument> {
    vec![
        ContextualDocument {
            document: Document::from_parts("Intro to Integrals", INTEGRALS_NOTE),
            tags: vec![Tag {
                name: "Math".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("SIMD Intro", SIMD_NOTE),
            tags: vec![Tag {
                name: "Computer Science".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("Discrete Math", DISCRETE_MATH_NOTE),
            tags: vec![Tag {
                name: "Math".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("Statistics", STATS_NOTE),
            tags: vec![Tag {
                name: "Math".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("Reactivity systems", REACTIVITY_NOTE),
            tags: vec![Tag {
                name: "Computer Science".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("Operating Systems", OS_NOTE),
            tags: vec![Tag {
                name: "Computer Science".to_string(),
                manual: true,
            }],
        },
    ]
}

#[tokio::test]
async fn test_tag_classifier() {
    use crate::note::{save_note, set_tags, Tag};
    use crate::workspace::{delete_workspace, load_workspace, unload_workspace};
    use kalosm::language::Document;
    use std::env::temp_dir;

    _ = tracing_subscriber::fmt::try_init();

    let temp = temp_dir();
    let workspace_path = temp.join("testing-classifier-workspace");
    let workspace = load_workspace(workspace_path);
    let title = "test-note".to_string();
    let text = "Computer science is the study of computation and its applications.".to_string();
    let tags = vec![Tag {
        name: "tag1".to_string(),
        manual: true,
    }];
    save_note(title.clone(), text.clone(), workspace).await;
    set_tags(title.clone(), tags.clone(), workspace)
        .await
        .unwrap();
    let title2 = "my-other-test-note".to_string();
    let text2 = "Economics describes a process for distributing limited resources in a way that maximizes overall satisfaction of a group of people.".to_string();
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
    save_note(title2.clone(), text2.clone(), workspace).await;
    set_tags(title2.clone(), tags2.clone(), workspace)
        .await
        .unwrap();

    {
        let workspace = crate::workspace::get_workspace_ref(workspace);
        let workspace = &*workspace;
        let _ = TagClassifier::new(
            workspace,
            &[
                ContextualDocument {
                    document: Document::from_parts(title.clone(), text.clone()),
                    tags: tags.clone(),
                },
                ContextualDocument {
                    document: Document::from_parts(title2.clone(), text2.clone()),
                    tags: tags2.clone(),
                },
            ],
            |_| {},
        )
        .await;
    }
    delete_workspace(workspace);
    unload_workspace(workspace);
}
