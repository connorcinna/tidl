use clap::Parser;
use reqwest::{Response, Client};
use terminal_menu::{menu, button, run, mut_menu};
use std::collections::HashMap;
use std::error::Error;
use serde_json::Value;

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

fn display_download_options(opts: &Vec<String>) -> Result<String, Box<dyn Error>>
{
    let menu = menu(
        opts.iter().map(|n| button(format!("{}", n))).collect()
    );
    run(&menu);
    if mut_menu(&menu).canceled()
    {
        println!("Canceled!");
        return Err("Canceled".into());
    }
    return Ok(String::from(mut_menu(&menu).selected_item_name()));
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
    //track_search: vec of strings containing IDs of songs like "932493797"
    let track_search = tidal_rs::search_get_track(&client, args.query.clone()).await;
    //tracks: hashmap containing the song name deep down
    let mut tracks : Vec<Result<HashMap<String, Value>, Box<dyn Error>>> = Vec::new();
    for track in &track_search
    {
        tracks.push(tidal_rs::get_track_by_id(&client, track.clone(), s!("US")).await);
    }
    //get the track name, artist, etc.
    //need to get album cover too.. this is a lot of requests
    let mut track_names : Vec<String> = Vec::new();
    for s in tracks
    {
        match s
        {
            Ok(track_map) =>
            {
                track_names.push(track_map
                    .get("data")
                    .unwrap()
                    .get("attributes")
                    .unwrap()
                    .get("title")
                    .unwrap()
                    .to_string());
            }
            Err(_) =>
            {
                //println!("{e}");
            }
        }
    }
    let mut track_id_to_songname: HashMap<String, String> = HashMap::new();
    for (index, _) in track_names.iter().enumerate()
    {
        track_id_to_songname.insert(track_names[index].clone(), track_search[index].clone());
    }
    let selected_song_name = display_download_options(&track_names);
    match selected_song_name
    {
        Ok(song) =>
        {
            let filename = crate::util::generate_filename(song.clone(), s!(".flac"));
            //TODO: figure out the filetype from the url if possible
            let track_id = track_id_to_songname.get(&song).unwrap();
            let track_url = tidal_rs::dl_get_track_url(&client, track_id.clone(), &mut auth).await;
            match crate::util::download_file(&client, track_url, format!("{0}/{1}", args.destination, filename.clone())).await
            {
                Ok(()) =>
                {
                    println!("File successfully downloaded at {0}/{1}", args.destination, filename.clone());
                }
                Err(e) =>
                {
                    eprintln!("{e}");
                }
            }
        }
        Err(e) =>
        {
            eprintln!("{e}");
        }
    }
}
