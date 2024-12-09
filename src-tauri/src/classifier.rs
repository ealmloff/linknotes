/*!
# Natural Language Processing and Classification Framework
# Description: This Rust module integrates components for processing, embedding, and classifying textual data using the `kalosm` crate. It provides functionalities that are particularly suited for building applications involving natural language understanding and context-aware classification. Below is a summary of the key features and components:

## Programmers: Ealmoff, Suhaan
## Date created: 2024-10-14 
## Last modified: 2024-12-08
## Revision: -> Trained model with new notes and tags

## Key Features

### Text Chunking and Sentence Splitting
The `chunk_text` function processes raw text, splitting it into meaningful chunks and sentences based on bullet points, numbered lists, and sentence boundaries. This segmentation allows efficient downstream operations like embedding and classification.

### Embedding and BERT Integration
Leveraging the `kalosm::language` module, the code uses BERT-based embeddings (`BertSpace`) to represent textual data in a high-dimensional vector space. These embeddings capture semantic and contextual information, which is critical for classification tasks.

### Classification System
- **Tag Classification**: Implements a `TagClassifier` for associating textual data with predefined tags. The classifier is trained using embeddings and corresponding tag associations.
- **Data Handling**: Utilizes `ClassificationDatasetBuilder` to prepare data for training a classifier.
- **Model Configuration**: Supports hyperparameter customization such as learning rate, batch size, and training epochs.

### Contextual Document Management
The code includes a framework for handling documents with associated metadata (`ContextualDocument`) and tags. Documents are processed and embedded using a unified workflow.

### Note Handling
Includes constants referencing various notes stored in text files. These notes are used as sample input data for the classification system. Topics include Math, Computer Science, Philosophy, and more.

## Code Structure

### Imports
- **Natural Language Processing**: Includes modules for embedding, chunking, and device acceleration (`kalosm::language`).
- **Learning and Classification**: Provides tools for dataset preparation and classification (`kalosm_learning`).
- **Synchronization Utilities**: Utilizes `OnceLock` for thread-safe initialization of shared resources.

### Core Functions
- **`chunk_text`**: Processes input text into chunks and sentences, returning ranges of character indices for each segment.
- **`default_documents`**: Initializes a set of sample `ContextualDocument` instances with associated tags.

### Class Definitions
- **`TagClassifier`**: Encapsulates the logic for training and using a tag-based text classifier. Includes methods for classifier initialization (`new`) and text classification (`classify`).

## Usage Example
The module supports testing and demonstrates its functionality through the `test_chunk_text` function, which validates the correctness of the `chunk_text` logic.

## Applications
This code is designed for natural language processing applications such as:
- Text summarization
- Contextual tagging
- Document classification
- Semantic search and retrieval
*/





/// importing modules from the `kalosm` crate related to natural language processing tasks. 
/// It includes imports for features such as accelerated
/// device availability, BertSpace, Chunker, Document, EmbedderExt, Embedding, SentenceChunker, and
/// DefaultSentenceChunker. These modules are likely used for tasks such as text chunking, document
/// processing, and embedding operations in natural language processing applications.
use kalosm::language::{
    accelerated_device_if_available, BertSpace, Chunker, DefaultSentenceChunker, Document,
    EmbedderExt, Embedding, SentenceChunker,
};
/// Importing module to build a classification dataset, configure a classifier, and track the progress of the
/// classifier.
use kalosm_learning::{
    ClassificationDatasetBuilder, Classifier, ClassifierConfig, ClassifierProgress,
};
/// The `OnceLock` struct is used for synchronization in Rust to ensure
/// that a certain block of code is only executed once, even in a multi-threaded context.
use std::{ops::Range, sync::OnceLock};

/// Creating a crate and defining a module that includes the `bert`, `note`, and `workspace` modules. 
/// It also includes a struct `ContextualDocument` and an
/// enum `Tag` from the `note` module. 
///This code is setting up the structure and dependencies for a workspace in a Rust project.
use crate::{
    bert,
    note::{ContextualDocument, Tag},
    workspace::Workspace,
};

/// The `chunk_text` function in Rust splits text into chunks based on bullet points and sentence
/// boundaries.
/// 
/// Arguments:
/// 
/// * `text`: The `chunk_text` function takes a text input and splits it into chunks based on certain
/// criteria. It first splits the text based on bullet points or numbered lists, then further splits
/// each chunk into individual sentences.
/// 
/// Returns:
/// 
/// The `chunk_text` function returns a vector of `Range<usize>`, which represents the ranges of text
/// segments after processing based on bullet points and sentence splitting.
pub(crate) fn chunk_text(text: &str) -> Vec<Range<usize>> {
    // First split based on bullet points
    let mut segments = Vec::new(); // Vector to store the ranges of text segments
    let mut last_idx = 0; // Index to keep track of the last processed index
    let mut iter = text.char_indices(); // Iterator over the characters in the text
    let mut add_line = |text: &str, range: Range<usize>, last_idx: &mut usize| { // Function to add a line to the segments
        let line = &text[range.clone()]; // Get the line based on the range
        let mut line_chars = line.char_indices().skip_while(|(_, c)| c.is_whitespace()); // Iterator over the characters in the line
        match line_chars.next() { // Match on the first character in the line
            Some((idx, '-')) => { // If the first character is a hyphen
                let mut idx = idx; // Initialize the index
                while let Some((new_idx, c)) = line_chars.next() { // Iterate over the characters in the line
                    idx = new_idx; // Update the index
                    if !c.is_whitespace() { // If the character is not whitespace
                        break;
                    }
                }
                let range = range.start + idx..range.end; // Create a range based on the index
                *last_idx = range.end; // Update the last index
                segments.push(range); // Add the range to the segments
            }
            Some((idx, c)) if c.is_numeric() => { // If the first character is numeric
                let mut idx = idx; // Initialize the index
                while let Some((new_idx, c)) = line_chars.next() { // Iterate over the characters in the line
                    idx = new_idx; // Update the index
                    if !(c.is_numeric() || c == ')' || c == '.' || c.is_whitespace()) { // If the character is not numeric, a closing parenthesis, a period, or whitespace
                        break;
                    }
                }
                let range = range.start + idx..range.end; // Create a range based on the index
                *last_idx = range.end; // Update the last index
                segments.push(range); // Add the range to the segments
            }
            _ => {}
        }
    };
    loop {
        match iter.next() { // Match on the next character in the text
            Some((idx, '\n')) => { // If the character is a newline
                add_line(&text, last_idx..idx, &mut last_idx); // Add the line to the segments
            }
            Some((_, _)) => continue, // If the character is not a newline, continue to the next character
            None => break, // If there are no more characters, break out of the loop
        }
    }
    add_line(&text, last_idx..text.len(), &mut last_idx); // Add the last line to the segments
    if last_idx < text.len() { // If there are characters remaining in the text
        let remaining = &text[last_idx..]; // Get the remaining text
        let mut idx = 0; // Initialize the index
        let mut iter = remaining.char_indices(); // Iterator over the characters in the remaining text
        while let Some((new_idx, c)) = iter.next() { // Iterate over the characters in the remaining text
            idx = new_idx; // Update the index
            if !c.is_whitespace() { // If the character is not whitespace
                break;
            }
        }
        let mut back_idx = remaining.len(); // Initialize the back index
        let mut iter = remaining.char_indices().rev(); // Reverse iterator over the characters in the remaining text
        while let Some((new_idx, c)) = iter.next() { // Iterate over the characters in the remaining text
            if !c.is_whitespace() {
                break;
            }
            back_idx = new_idx;
        }
        segments.push(last_idx + idx..last_idx + back_idx);
    }
    let mut ranges = Vec::new();
    /// The above Rust code is iterating over a collection of segment ranges and extending a vector of
    /// ranges with the result of splitting sentences from the text within each segment range. It uses a
    /// `SentenceChunker` to split sentences and then maps the resulting sentence ranges to adjust their
    /// start and end positions based on the segment range they belong to.
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
    ) -> anyhow::Result<Self> {
        static DEFAULT_EMBEDDED_DOCUMENTS: OnceLock<Vec<(String, Embedding<BertSpace>)>> =
            OnceLock::new();
        if DEFAULT_EMBEDDED_DOCUMENTS.get().is_none() {
            let mut cached_doc_tags = Vec::new();
            let mut cached_doc_text = Vec::new();
            let default_docs = default_documents();
            for document in &default_docs {
                let text = document.document.body();
                let chunks = chunk_text(text);
                for chunk in &chunks {
                    cached_doc_tags.push(document.tags.clone());
                    cached_doc_text.push(&text[chunk.clone()]);
                }
            }

            let bert = bert().await.unwrap();
            let embeddings = bert.embed_batch(&cached_doc_text).await.unwrap();

            let mut tag_embeddings = Vec::new();
            for (tags, embedding) in cached_doc_tags.into_iter().zip(embeddings.into_iter()) {
                for tag in tags {
                    tag_embeddings.push((tag.name, embedding.clone()));
                }
            }

            DEFAULT_EMBEDDED_DOCUMENTS.set(tag_embeddings).unwrap();
        }
        let mut tagged_documents = DEFAULT_EMBEDDED_DOCUMENTS.get().unwrap().clone();
        let mut new_document_tags = Vec::new();
        let mut new_document_text = Vec::new();
        for document in documents {
            let text = document.document.body();
            let chunks = chunk_text(text);
            for chunk in &chunks {
                new_document_tags.push(document.tags.clone());
                new_document_text.push(&text[chunk.clone()]);
            }
        }

        let bert = bert().await.unwrap();
        let embeddings = bert.embed_batch(&new_document_text).await.unwrap();
        for (tags, embedding) in new_document_tags.into_iter().zip(embeddings.into_iter()) {
            for tag in tags {
                tagged_documents.push((tag.name, embedding.clone()));
            }
        }

        let epochs = 2;
        let learning_rate = 0.003;
        let batch_size = 50;
        let mut dataset = ClassificationDatasetBuilder::<u32>::new();
        for (tag, embedding) in tagged_documents {
            let id = workspace.get_tag_id(tag.as_str());
            let embedding_vec = embedding.to_vec();
            dataset.add(embedding_vec, id);
        }

        // After we have added all of the tags to the dataset, get the class count and train the model
        let class_count = workspace.tag_count() as u32;
        let config = ClassifierConfig::new().classes(class_count);
        let device = accelerated_device_if_available().unwrap();
        let classifier = Classifier::new(&device, config).unwrap();

        let device = accelerated_device_if_available().unwrap();
        let dataset = dataset.build(&device).unwrap();
        classifier.train(&dataset, epochs, learning_rate, batch_size, progress)?;

        Ok(Self { classifier })
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

/// The above code in Rust is defining constants that store the content of text files located in the
/// "classifier-notes" directory. Each constant corresponds to a specific note related to a topic such
/// as integrals, SIMD, discrete math, reactivity, operating systems, statistics, history, science,
/// physics, and philosophy. The `include_str!` macro is used to include the content of the text files
/// as string literals in the constants. This allows the program to access and use the content of these
/// notes during runtime.
const INTEGRALS_NOTE: &str = include_str!("./classifier-notes/integrals.note");
const SIMD_NOTE: &str = include_str!("./classifier-notes/simd.note");
const DISCRETE_MATH_NOTE: &str = include_str!("./classifier-notes/discrete-math.note");
const REACTIVITY_NOTE: &str = include_str!("./classifier-notes/reactivity.note");
const OS_NOTE: &str = include_str!("./classifier-notes/os.note");
const STATS_NOTE: &str = include_str!("./classifier-notes/stats.note");
const HISTORY_NOTE: &str = include_str!("./classifier-notes/history.note");
const SCIENCE_NOTE: &str = include_str!("./classifier-notes/science.note");
const PHYSICS_NOTE: &str = include_str!("./classifier-notes/physics.note");
const PHILOSOPHY_NOTE: &str = include_str!("./classifier-notes/philosophy.note");

/// The function `default_documents` returns a vector of `ContextualDocument` instances with associated
/// tags.
/// 
/// Returns:
/// 
/// A vector of `ContextualDocument` structs is being returned. Each `ContextualDocument` contains a
/// `Document` and a vector of `Tag`s. The documents include topics such as "Intro to Integrals", "SIMD
/// Intro", "Discrete Math", "Statistics", "Reactivity systems", "Operating Systems", "History",
/// "Philosophy", "Science", and "Physics
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
        ContextualDocument {
            document: Document::from_parts("History", HISTORY_NOTE),
            tags: vec![Tag {
                name: "History".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("Philosophy", PHILOSOPHY_NOTE),
            tags: vec![Tag {
                name: "Philosophy".to_string(),
                manual: true,
                }],
        },
        ContextualDocument {
            document: Document::from_parts("Science", SCIENCE_NOTE),
            tags: vec![Tag {
                name: "Science".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("Physics", PHYSICS_NOTE),
            tags: vec![Tag {
                name: "Physics".to_string(),
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
    save_note(title.clone(), text.clone(), workspace)
        .await
        .unwrap();
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
    save_note(title2.clone(), text2.clone(), workspace)
        .await
        .unwrap();
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
        .await
        .unwrap();
    }
    delete_workspace(workspace);
    unload_workspace(workspace);
}
