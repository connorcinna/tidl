use reqwest::Client;
use std::error::Error;
use crate::structs::TidalError;
use futures_util::StreamExt;
use std::fmt::Write;
use tokio::io::AsyncWriteExt;
use tokio::fs::File;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

macro_rules! s {
    ($s:expr) => { $s.to_string() }
}

pub fn sanitize_url(mut url: String) -> String
{
    url = url.replace(' ', "%20");
    url = url.replace('\'', "");
    url = url.replace('\"', "");
    url
}

pub fn sanitize_filename(mut name: String) -> String
{
    name = name.replace("\"", "");
    name = name.replace(" ", "_");
    name
}

pub async fn trim_last_char(value: &str) -> &str
{
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}

pub fn generate_filename(query: String, filetype: String) -> String
{
    let date_string = chrono::offset::Local::now();
    let filename = format!("{0}_{1}.{2}", query, date_string, filetype);
    sanitize_filename(filename)
}

pub fn which_filetype(url: String) -> String
{
    for s in vec!["flac", "mp4"]
    {
        if url.contains(s)
        {
            return s!(s);
        }
    }
    //default to mp4
    s!(".mp4")
}

//TODO when the database gets introduced, attach metrics to this function
//date of download, filesize, user, etc
//Also check if this url has been downloaded before by this user
pub async fn download_file(client: &Client, url: String, dest: String) -> Result<(), Box<dyn Error>>
{
    let response = client
        .get(url)
        .send()
        .await?;
    match response.error_for_status()
    {
        Ok(res) =>
        {
            let total_size = res.content_length().expect("res did not include Content-Length");
            let mut stream = res.bytes_stream();
            let pb = ProgressBar::new(total_size);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                .progress_chars("#>-"));

            let mut downloaded: u64 = 0;
            let mut file = File::create(dest).await?;
            while let Some(chunk) = stream.next().await
            {
                let chunk = chunk.unwrap();
                let bytes = &chunk.as_ref();
                file.write_all(bytes).await?;
                downloaded += bytes.len() as u64;
                pb.set_position(downloaded);
            }
            Ok(())
        }
        Err(e) =>
        {
            Err(Box::new(TidalError(e.to_string())))
        }
    }
}
