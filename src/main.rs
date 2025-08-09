use clap::Parser;
use reqwest::{Response, Client};

extern crate tidal_rs;
use tidal_rs::{structs, util, user};

macro_rules! s {
    ($s:expr) => { $s.to_string() }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args
{
    /// Query to pass to Tidal
    #[arg(short, long)]
    query: String,
    /// Location to put downloaded files
    #[arg(short, long, default_value_t = String::from("~/Downloads"))]
    destination: String
}

#[tokio::main]
async fn main()
{
    let args = Args::parse();
    let client : Client = Client::builder()
        .http1_title_case_headers()
        .build()
        .expect("Unable to build reqwest client");
    let mut auth = structs::DlBasicAuthResponse::default();
    let track_search = tidal_rs::search_get_track(&client, args.query.clone()).await;
    //get the track name, artist, etc.
    //need to get album cover too.. this is a lot of requests
    let track = tidal_rs::get_track_by_id(&client, track_search[0].clone(), s!("US")).await;
    let t = track.unwrap();
    println!("track: {:?}", t);
    let filename = crate::util::generate_filename(t
        .get("data")
        .expect("Unable to find data from the result")
        .get("attributes")
        .expect("Unable to find attributes from the result")
        .get("title")
        .expect("Unable to find title from the result")
        .to_string(), s!(".flac"));
    //TODO: figure out the filetype from the url if possible
    let track_url = tidal_rs::dl_get_track_url(&client, track_search[0].clone(), &mut auth).await;
    match crate::util::download_file(&client, track_url, format!("{0}/{1}", args.destination, filename.clone())).await
    {
        Ok(()) =>
        {
            println!("File successfully downloaded at {0}", filename.clone());
        }
        Err(e) =>
        {
            eprintln!("{e}");
        }
    }
}
