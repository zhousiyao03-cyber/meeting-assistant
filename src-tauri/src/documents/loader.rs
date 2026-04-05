use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize)]
pub struct LoadedDocument {
    pub filename: String,
    pub content: String,
    pub format: String,
}

/// Load a document file. Supports .md, .txt, .pdf (text extraction only).
pub fn load_document(path: &Path) -> Result<LoadedDocument> {
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let content = match ext.as_str() {
        "md" | "txt" | "text" => fs::read_to_string(path)?,
        "pdf" => {
            // Basic PDF text extraction - in production use a proper PDF library
            // For now, fallback to reading raw bytes and extracting text-like content
            let bytes = fs::read(path)?;
            extract_pdf_text(&bytes).unwrap_or_else(|| {
                "[PDF 文件 - 无法提取文本，请转换为 Markdown 或纯文本格式]".into()
            })
        }
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", ext)),
    };

    Ok(LoadedDocument {
        filename,
        content,
        format: ext,
    })
}

/// Very basic PDF text extraction (looks for text between BT/ET markers).
/// For production, use `pdf-extract` or `lopdf` crate.
fn extract_pdf_text(_bytes: &[u8]) -> Option<String> {
    // Placeholder: return None to trigger the fallback message.
    // A real implementation would use a PDF parsing library.
    None
}

/// Chunk a document into paragraphs for context window management.
#[allow(dead_code)]
pub fn chunk_document(content: &str, max_chunk_chars: usize) -> Vec<String> {
    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mut chunks = Vec::new();
    let mut current = String::new();

    for para in paragraphs {
        if current.len() + para.len() > max_chunk_chars && !current.is_empty() {
            chunks.push(current.clone());
            current.clear();
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Select the most relevant chunk based on keyword overlap with the transcript.
#[allow(dead_code)]
pub fn select_relevant_chunk(chunks: &[String], transcript: &str) -> String {
    if chunks.is_empty() {
        return String::new();
    }
    if chunks.len() == 1 {
        return chunks[0].clone();
    }

    // Simple keyword scoring: count how many transcript words appear in each chunk
    let transcript_words: Vec<&str> = transcript.split_whitespace().collect();
    let mut best_idx = 0;
    let mut best_score = 0;

    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_lower = chunk.to_lowercase();
        let score = transcript_words
            .iter()
            .filter(|w| w.len() > 2 && chunk_lower.contains(&w.to_lowercase()))
            .count();
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    chunks[best_idx].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_document() {
        let doc = "Para one.\n\nPara two.\n\nPara three which is longer.";
        let chunks = chunk_document(doc, 30);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_select_relevant() {
        let chunks = vec![
            "数据库设计和 schema 优化方案".into(),
            "前端 React 组件的测试策略".into(),
        ];
        let transcript = "我们来讨论一下数据库的 schema 怎么设计";
        let selected = select_relevant_chunk(&chunks, transcript);
        assert!(selected.contains("数据库"));
    }
}
