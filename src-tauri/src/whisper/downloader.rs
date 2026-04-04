use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const SENSEVOICE_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17.tar.bz2";
const SENSEVOICE_ARCHIVE: &str = "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17.tar.bz2";
const SENSEVOICE_DIR: &str = "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17";

const VAD_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/silero_vad.onnx";
const VAD_FILENAME: &str = "silero_vad.onnx";

/// Returns the path to the models directory (~/.meeting-assistant/models/).
pub fn models_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let dir = home.join(".meeting-assistant").join("models");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the path to the SenseVoice model directory if all required files exist.
pub fn model_path() -> Result<Option<PathBuf>> {
    let dir = models_dir()?.join(SENSEVOICE_DIR);
    let model_file = dir.join("model.int8.onnx");
    let tokens_file = dir.join("tokens.txt");
    let vad_file = dir.join(VAD_FILENAME);

    if model_file.exists() && tokens_file.exists() && vad_file.exists() {
        Ok(Some(dir))
    } else {
        Ok(None)
    }
}

/// Download the SenseVoice model + Silero VAD, calling `on_progress(bytes_downloaded, total_bytes)`.
pub async fn download_model<F>(on_progress: F) -> Result<PathBuf>
where
    F: Fn(u64, u64) + Send + 'static,
{
    let model_dir = models_dir()?.join(SENSEVOICE_DIR);

    // Check if already fully downloaded
    if model_dir.join("model.int8.onnx").exists()
        && model_dir.join("tokens.txt").exists()
        && model_dir.join(VAD_FILENAME).exists()
    {
        return Ok(model_dir);
    }

    fs::create_dir_all(&model_dir)?;

    // Download SenseVoice archive if model file doesn't exist
    if !model_dir.join("model.int8.onnx").exists() {
        let archive_path = models_dir()?.join(SENSEVOICE_ARCHIVE);
        download_file(SENSEVOICE_URL, &archive_path, &on_progress).await?;
        extract_tar_bz2(&archive_path, &models_dir()?)?;
        let _ = fs::remove_file(&archive_path);
    }

    // Download Silero VAD if it doesn't exist
    let vad_dest = model_dir.join(VAD_FILENAME);
    if !vad_dest.exists() {
        download_file(VAD_URL, &vad_dest, &on_progress).await?;
    }

    // Verify required files
    if !model_dir.join("model.int8.onnx").exists() {
        return Err(anyhow::anyhow!(
            "model.int8.onnx not found after extraction"
        ));
    }

    Ok(model_dir)
}

async fn download_file<F>(url: &str, dest: &PathBuf, on_progress: &F) -> Result<()>
where
    F: Fn(u64, u64),
{
    let tmp_dest = dest.with_extension("tmp");

    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Download failed with status {}: {}",
            resp.status(),
            url
        ));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = fs::File::create(&tmp_dest)?;
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    let result: Result<()> = async {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            on_progress(downloaded, total);
        }
        Ok(())
    }
    .await;

    if let Err(e) = result {
        let _ = fs::remove_file(&tmp_dest);
        return Err(e);
    }

    if total > 0 && downloaded != total {
        let _ = fs::remove_file(&tmp_dest);
        return Err(anyhow::anyhow!(
            "Download incomplete: got {} of {} bytes",
            downloaded,
            total
        ));
    }

    fs::rename(&tmp_dest, dest)?;
    Ok(())
}

fn extract_tar_bz2(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<()> {
    use std::io::BufReader;

    let file = fs::File::open(archive_path)?;
    let reader = BufReader::new(file);

    // bz2 decode
    let bz2_reader = bzip2::read::BzDecoder::new(reader);

    // tar extract
    let mut archive = tar::Archive::new(bz2_reader);
    archive.unpack(dest_dir)?;

    eprintln!(
        "[downloader] Extracted {:?} to {:?}",
        archive_path, dest_dir
    );
    Ok(())
}
