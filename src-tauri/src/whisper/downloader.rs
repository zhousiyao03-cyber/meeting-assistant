use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin";
const MODEL_FILENAME: &str = "ggml-medium.bin";

/// Returns the path to the models directory (~/.meeting-assistant/models/).
pub fn models_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let dir = home.join(".meeting-assistant").join("models");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the path to the model file if it exists.
pub fn model_path() -> Result<Option<PathBuf>> {
    let path = models_dir()?.join(MODEL_FILENAME);
    if path.exists() {
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

/// Download the Whisper model, calling `on_progress(bytes_downloaded, total_bytes)`.
pub async fn download_model<F>(on_progress: F) -> Result<PathBuf>
where
    F: Fn(u64, u64) + Send + 'static,
{
    let dest = models_dir()?.join(MODEL_FILENAME);
    if dest.exists() {
        return Ok(dest);
    }

    let client = reqwest::Client::new();
    let resp = client.get(MODEL_URL).send().await?;
    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = fs::File::create(&dest)?;
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    Ok(dest)
}
