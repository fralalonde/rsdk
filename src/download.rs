use std::path::Path;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn download_with_progress(url: &str, output_path: &Path, client: &reqwest_middleware::ClientWithMiddleware) -> Result<()> {
    // Send the GET request
    let response = client.get(url).send().await
        .context("Failed to GET")?;

    // Check if the response is successful
    if !response.status().is_success() {
        anyhow::bail!("Failed to download file: HTTP {}", response.status());
    }

    // Get the total size from the Content-Length header
    let total_size = response
        .content_length()
        .ok_or_else(|| anyhow::anyhow!("Failed to get content length from response"))?;

    // Create the progress bar
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-")
    );

    // Open the output file
    let mut file = File::create(output_path).await
        .context("Failed to create output file")?;

    // Wrap the response body stream
    let mut stream = response.bytes_stream();

    // Download the file, updating the progress bar
    while let Some(chunk) = stream.next().await {
        let data = chunk.context("Error while downloading file")?;
        file.write_all(&data).await.context("Error while writing to file")?;
        pb.inc(data.len() as u64);
    }

    pb.finish_with_message("Download completed");

    Ok(())
}
