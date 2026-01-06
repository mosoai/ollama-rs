use ollama_rs::{generation::embeddings::request::GenerateEmbeddingsRequest, Ollama};

// NOTE: Update the URL below to point to your Ollama instance
const OLLAMA_URL: &str = "http://localhost:11434";

#[tokio::test]
async fn test_embeddings_generation() {
    let ollama = Ollama::try_new(OLLAMA_URL).expect("Invalid Ollama URL");

    let res = ollama
        .generate_embeddings(GenerateEmbeddingsRequest::new(
            "qwen3-embedding:0.6b".to_string(),
            "Why is the sky blue".into(),
        ))
        .await
        .unwrap();

    dbg!(res);
}

#[tokio::test]
async fn test_batch_embeddings_generation() {
    let ollama = Ollama::try_new(OLLAMA_URL).expect("Invalid Ollama URL");

    let res = ollama
        .generate_embeddings(GenerateEmbeddingsRequest::new(
            "qwen3-embedding:0.6b".to_string(),
            vec!["Why is the sky blue?", "Why is the sky red?"].into(),
        ))
        .await
        .unwrap();

    dbg!(res);
}

#[tokio::test]
async fn test_embeddings_generation_with_dimensions() {
    let ollama = Ollama::try_new(OLLAMA_URL).expect("Invalid Ollama URL");

    let res = ollama
        .generate_embeddings(
            GenerateEmbeddingsRequest::new(
                "qwen3-embedding:0.6b".to_string(),
                "Why is the sky blue".into(),
            )
            .dimensions(768),
        )
        .await
        .unwrap();

    dbg!(res);
}
