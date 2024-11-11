use kalosm::language::{accelerated_device_if_available, Document, EmbedderExt};
use kalosm_learning::{
    Classifier, ClassifierConfig, ClassifierProgress, TextClassifierDatasetBuilder,
};

use crate::{
    bert,
    note::{ContextualDocument, Tag},
    workspace::Workspace,
};

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
            for tag in &document.tags {
                let id = { workspace.get_tag_id(tag.name.as_str()) };
                tagged_documents.push((document.document.body(), id));
            }
        }
        let class_count = { workspace.tag_count() as u32 };
        let config = ClassifierConfig::new().classes(class_count);
        let classifier = Classifier::new(&device, config).unwrap();

        let device = accelerated_device_if_available().unwrap();
        let epochs = 10;
        let learning_rate = 0.003;
        let batch_size = 5;
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

    pub async fn classify(&self, workspace: &Workspace, document: &ContextualDocument) -> Tag {
        let bert = bert().await.unwrap();
        let embedding = bert.embed(document.document.body()).await.unwrap();
        let embedding = embedding.to_vec();
        let classes = self.classifier.run(&embedding).unwrap();
        let most_likely_tag = classes.top();
        let tag_id = workspace.get_tag_name(most_likely_tag);
        Tag {
            name: tag_id.to_string(),
            manual: false,
        }
    }
}

const INTEGRALS_BODY: &str = r#"# Introduction to Integrals

Integrals are one of the fundamental concepts of calculus, representing the accumulation of quantities over time, distance, or any other continuous variable. The concept emerged from the ancient Greek method of exhaustion, though modern integral calculus was primarily developed by Newton and Leibniz in the 17th century.

## Definite and Indefinite Integrals

An indefinite integral, also known as an antiderivative, represents a family of functions whose derivative equals the integrand. When we write ∫f(x)dx, we're finding a function F(x) such that F'(x) = f(x). The solution always includes a constant of integration (+C) because derivatives of constants are zero.

Definite integrals, written as ∫[a to b]f(x)dx, calculate the precise area under a curve between two points. The fundamental theorem of calculus connects definite and indefinite integrals, stating that the definite integral equals the difference of the antiderivative evaluated at the endpoints: ∫[a to b]f(x)dx = F(b) - F(a).

## Integration Techniques

Basic integration relies on recognizing patterns and applying standard formulas. The power rule states that ∫xⁿdx = (xⁿ⁺¹)/(n+1) + C (for n ≠ -1). Integration by substitution (u-substitution) handles compositions of functions by temporarily replacing part of the integrand with a new variable.

Integration by parts, derived from the product rule, uses the formula ∫udv = uv - ∫vdu. This technique is particularly useful for products of functions like xex or xlnx. Partial fractions decomposition helps integrate rational functions by breaking them into simpler terms.

## Applications and Significance

Integrals have widespread applications across science and engineering. They calculate areas, volumes, work done by forces, fluid pressure, probability distributions, and much more. In physics, definite integrals compute displacement from velocity, or work from force. In statistics, they determine probabilities in continuous distributions.

## Connection to Geometry

Geometrically, definite integrals represent areas under curves, but this interpretation extends to more complex scenarios. Double integrals compute volumes under surfaces, while line integrals calculate quantities along paths. Surface integrals extend this to curved surfaces in three dimensions.

The concept of a Riemann sum underlies the definition of definite integrals, approximating areas by dividing regions into rectangles and taking the limit as their width approaches zero. This connects to the ancient Greek method of exhaustion and provides a rigorous foundation for integration theory.

## Advanced Topics

Improper integrals handle integration over infinite intervals or around singularities. Multiple integrals extend integration to higher dimensions. Complex integration, developed by Cauchy, leads to powerful results in complex analysis and has applications in physics and engineering.

Modern integration theory includes Lebesgue integration, which extends the concept to more general functions and spaces. This theoretical framework is crucial in advanced mathematics, particularly in functional analysis and measure theory."#;

const SIMD_BODY: &str = r#"SIMD (Single Instruction, Multiple Data) acceleration is a parallel processing technique that significantly improves computational efficiency by performing the same operation on multiple data points simultaneously. Modern processors implement SIMD through specialized vector registers and instruction sets, such as Intel's SSE and AVX or ARM's NEON. For example, instead of adding two numbers sequentially, SIMD can add four, eight, or even sixteen pairs of numbers in a single clock cycle, depending on the hardware capabilities.
The key advantage of SIMD lies in its ability to accelerate data-parallel tasks common in multimedia processing, scientific computing, and graphics rendering. When processing image data, SIMD can manipulate multiple pixels simultaneously, making operations like brightness adjustment or color conversion much faster. Similarly, in audio processing, SIMD can handle multiple samples at once, significantly speeding up operations like filtering or mixing.
Programming with SIMD typically involves either using compiler auto-vectorization features or explicitly writing vectorized code using intrinsics - specialized functions that map directly to SIMD instructions. While auto-vectorization is convenient, manually optimized SIMD code often achieves better performance by allowing fine-grained control over data alignment, memory access patterns, and instruction scheduling. However, this comes with increased complexity and potential portability challenges across different architectures.
Common SIMD optimization patterns include loop unrolling, data alignment, and minimizing branching within vectorized code sections. Memory alignment is particularly crucial for SIMD performance, as misaligned memory access can negate the benefits of vectorization. Modern SIMD instruction sets also support sophisticated operations like shuffle, permute, and masked operations, enabling complex data manipulations within vector registers.
Despite its benefits, SIMD has limitations. Not all algorithms can be effectively vectorized, particularly those with complex control flow or data dependencies. SIMD programming requires careful consideration of data layout and memory access patterns. Additionally, different processor architectures support different SIMD instruction sets and vector widths, making it challenging to write portable optimized code."#;

const DISCRETE_MATH: &str = r#"Discrete mathematics forms the foundation of computer science and deals with mathematical structures that are fundamentally discrete (distinct and separate) rather than continuous. At its core, discrete mathematics studies countable mathematical objects and relationships, including integers, graphs, and statements in logic. Set theory serves as a crucial building block, dealing with collections of distinct objects and operations like unions, intersections, and complements. These concepts directly apply to database organization and algorithm design.
Boolean algebra, another fundamental component, works with true/false values and logical operators (AND, OR, NOT). This branch directly translates to computer circuit design and programming logic. Similarly, propositional and predicate logic provide frameworks for mathematical reasoning and proof techniques, including direct proofs, contradiction, and induction. Mathematical induction, in particular, offers a powerful method for proving statements about natural numbers and recursive structures.
Graph theory, a fascinating branch of discrete mathematics, studies networks of points (vertices) connected by lines (edges). This field has practical applications in social networks, transportation systems, and computer networks. Basic concepts include paths, cycles, trees, and connectivity. More advanced topics cover graph coloring, matching problems, and network flows, which find applications in scheduling, resource allocation, and optimization problems.
Number theory explores the properties of integers and their relationships. Key concepts include divisibility, prime numbers, greatest common divisors, and modular arithmetic. These principles are fundamental to cryptography and security systems. The Chinese Remainder Theorem and Euler's totient function play crucial roles in modern encryption methods like RSA.
Combinatorics focuses on counting techniques and arrangement patterns. This includes permutations, combinations, and the principle of inclusion-exclusion. More advanced topics cover generating functions, recurrence relations, and Burnside's lemma. These concepts are essential for analyzing algorithm efficiency, probability calculations, and solving complex counting problems.
Relations and functions in discrete mathematics differ from their continuous counterparts. Important concepts include equivalence relations, partial orders, and function properties like injectivity and surjectivity. These ideas help in understanding database relationships, algorithm analysis, and abstract data types in computer science.
Finite state machines and formal languages connect discrete mathematics to theoretical computer science. These models describe computational processes and help in designing software systems, compilers, and regular expressions. The study of automata theory bridges the gap between abstract mathematical concepts and practical computing applications.
The recursive nature of many discrete structures leads to the study of recurrence relations and difference equations. These mathematical tools help analyze algorithms, model population growth, and solve counting problems. Techniques for solving recurrence relations, including the characteristic equation method and generating functions, provide powerful analytical tools.
Coding theory and error detection/correction codes represent practical applications of discrete mathematics. These topics combine elements of linear algebra, polynomial arithmetic, and finite fields to create reliable communication systems. The mathematics behind error-correcting codes ensures data integrity in digital communications and storage systems.
Finally, discrete probability deals with random processes over discrete sample spaces. This includes concepts like random variables, expected value, and variance for discrete distributions. These principles are essential for analyzing algorithms, understanding data structures, and developing probabilistic algorithms."#;

const REACTIVITY_BODY: &str = r#"# Understanding Reactivity Systems: A Deep Dive

## Core Concepts and Implementation

Reactivity systems form the backbone of modern reactive programming frameworks like SolidJS, Vue, and MobX. At their core, these systems automatically track dependencies between data and the computations that depend on them, ensuring that when data changes, only the affected parts of the application update.

The fundamental building blocks of a reactivity system typically include signals (or observables), computations (or effects), and a dependency tracking mechanism. Signals represent atomic pieces of state that can change over time. Unlike traditional variables, signals notify their subscribers when their values change, enabling automatic updates throughout the application.

## Fine-Grained Reactivity

What sets systems like SolidJS apart is their implementation of fine-grained reactivity. Instead of relying on virtual DOM diffing like React, these systems create direct relationships between data and the DOM. When a signal changes, only the specific DOM elements that depend on that signal update, leading to superior performance characteristics.

Consider this implementation pattern:
```javascript
const count = createSignal(0);
const double = createMemo(() => count() * 2);
createEffect(() => console.log(double()));
```

In this example, changing `count` automatically updates `double` and triggers the effect, but only when necessary. The system maintains a dependency graph that tracks these relationships precisely.

## Dependency Tracking

The magic behind reactivity systems lies in their dependency tracking mechanism. During the execution of reactive computations, the system automatically records which signals are accessed. This is typically implemented using a global stack of currently executing computations and getter/setter interceptors on reactive values.

Modern reactivity systems often employ sophisticated techniques like:
- Batch updates to prevent cascading recomputations
- Topological sorting of dependencies to ensure updates occur in the correct order
- Memory management through weak references to prevent memory leaks
- Dependency cleanup to maintain system efficiency

## Comparison with Other Paradigms

Unlike traditional imperative programming where data flow must be manually managed, reactivity systems provide a declarative approach. They're similar to spreadsheet formulas – when an input cell changes, dependent formulas automatically recalculate. This paradigm significantly reduces the complexity of managing application state and UI updates.

## Performance Considerations

Reactivity systems achieve high performance through several key strategies:
1. Minimizing unnecessary recomputations through precise dependency tracking
2. Batching updates to reduce DOM operations
3. Avoiding intermediate representations like virtual DOM
4. Leveraging modern JavaScript features like Proxies for efficient tracking

## Common Patterns and Best Practices

When working with reactivity systems, several patterns emerge as best practices:
- Keeping reactive state as granular as possible
- Using derived values (computed properties) to cache complex calculations
- Carefully managing side effects to prevent infinite update loops
- Structuring dependencies to minimize the scope of updates

## Solid's Unique Approach

SolidJS specifically introduces some innovative concepts to the reactivity landscape:
- JSX compilation to direct DOM updates without Virtual DOM
- Reactive primitives that integrate seamlessly with JavaScript
- Component functions that run once rather than re-rendering
- Explicit reactive boundaries for better performance control

## Advanced Features

Modern reactivity systems often include advanced features like:
- Resource management for async operations
- Context-based dependency injection
- Reactive collections and deep reactivity
- Debug tooling for dependency visualization
- Lifecycle management for cleanup and disposal

## Challenges and Solutions

Common challenges in reactive systems include:
- Managing circular dependencies
- Handling asynchronous operations
- Controlling update granularity
- Memory management in long-running applications

Solutions often involve careful system design, such as:
- Cycle detection in dependency graphs
- Async computation primitives
- Manual control over computation scheduling
- Automatic cleanup of stale dependencies

## Future Directions

The field of reactive programming continues to evolve, with emerging trends like:
- Integration with WebAssembly for better performance
- Enhanced type system support
- Better tooling for dependency visualization
- Integration with server-side rendering
- Improved debugging capabilities

Understanding these concepts is crucial for building efficient, maintainable applications using modern reactive frameworks."#;

fn default_documents() -> Vec<ContextualDocument> {
    vec![
        ContextualDocument {
            document: Document::from_parts("Intro to Integrals", INTEGRALS_BODY.split_at(500).0),
            tags: vec![Tag {
                name: "Math".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("SIMD Intro", SIMD_BODY.split_at(500).0),
            tags: vec![Tag {
                name: "Computer Science".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("Discrete Math", DISCRETE_MATH.split_at(500).0),
            tags: vec![Tag {
                name: "Math".to_string(),
                manual: true,
            }],
        },
        ContextualDocument {
            document: Document::from_parts("Reactivity systems", REACTIVITY_BODY.split_at(500).0),
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
