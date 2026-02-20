use rand::Rng;
use tokio;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt}; 
use reqwest;
use open;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::fs;
use std::process::Command;
use clap::Parser;
use serde::{Serialize, Deserialize};

//spud --playlist [ID]

#[derive(Serialize, Deserialize)]
struct MyConfig {
    client_id: String,
    client_secret: String,
}
impl ::std::default::Default for MyConfig {
    fn default() -> Self { Self { client_id: "".into(), client_secret: "".into() } }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    playlist: String,
}

fn check_configuration()->(String, String){
    let cfg: MyConfig=confy::load("spud", None).expect("config failed");
    let mut client_id: String=cfg.client_id.to_string();
    let mut client_secret: String=cfg.client_secret.to_string();
    let mut update_flag: bool=false;
    if client_id=="" {
        println!("Missing Client ID in configuration");
        println!("Please enter your Spotify Client ID: ");
        let mut input=String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        client_id=input.trim().to_string();
        update_flag=true;
    }
    if client_secret=="" {
        println!("Missing Client Secret in configuration");
        println!("Please enter your Spotify Client Secret: ");
        let mut input=String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        client_secret=input.trim().to_string();
        update_flag=true;
    }
    if update_flag {
        let new_cfg= MyConfig{
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
        };
        confy::store("spud", None, new_cfg).expect("failed updating config")
    }
    (client_id, client_secret)
}

fn download_songs() -> std::io::Result<()> {
    fs::create_dir_all("songs")?;
    let output = Command::new("yt-dlp")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("-o")
        .arg("songs/%(artist)s - %(title)s.%(ext)s")
        .arg("--no-embed-thumbnail")
        .arg("--add-metadata")
        .arg("--extractor-args")
        .arg("youtube:player_client=android")
        .arg("-a")
        .arg("songs.txt") 
        .spawn()?        
        .wait_with_output()?;

    if output.status.success() {
        println!("Download completed successfully!");
    } else {
        eprintln!("Error during download: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

fn generate_random_string(length: u32)->String{
    let mut result=String::new();
    let char="ABCDEFGHIJKLMOPQRSTUVWXYZabcdefghijklmopqrstuvwxyz1234567890".as_bytes();
    for _ in 0..length{
        result.push(char[rand::thread_rng().gen_range(0..char.len())] as char);
    }
    result
}

async fn get_code(client_id: &str) -> Result<String, Box<dyn std::error::Error>>{
    println!("here");
    let perm_state=generate_random_string(16);
    let spotify_url = format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
        client_id,
        "http://127.0.0.1:8888/callback",
        "user-read-private%20user-read-email",
        perm_state
    );
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    let path = "";
    match open::that(&spotify_url) {
        Ok(()) => println!("Opened '{}' successfully.", path),
        Err(err) => eprintln!("An error occurred when opening '{}': {}", path, err),
    }
    //create TCP connection server
    let (mut socket, _)=listener.accept().await?;
    let mut buf=[0;4096];
    let n = socket.read(&mut buf).await?;
    let request_text = String::from_utf8_lossy(&buf[..n]);
    let code = request_text
        .split("code=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .and_then(|s| s.split(' ').next())
        .unwrap_or("");
    println!("Captured Code: {}", code);
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<h1>Logged in!</h1><p>You can close this tab.</p>";
    socket.write_all(response.as_bytes()).await?;
    Ok(code.to_string())
}

async fn get_authentication_token(client_id: &str,client_secret: &str) -> Result<String, Box<dyn std::error::Error>>{
    let code=get_code(&client_id).await?;
    let client=reqwest::Client::new();
    let token_url = "https://accounts.spotify.com/api/token";
    let params=[
        ("code",code.as_str()),
        ("redirect_uri","http://127.0.0.1:8888/callback"),
        ("grant_type","authorization_code"),
    ];
    let res=client.post(token_url).form(&params).basic_auth(client_id,Some(client_secret)).send().await?;
    // println!("{:?}",auth_token);
    let json: serde_json::Value=res.json().await?;
    let auth_token=&json["access_token"];
    println!("{:?}",auth_token);
    Ok(auth_token.as_str().ok_or("err")?.to_string())
}

#[tokio::main]
async fn main()-> Result<(),Box<dyn std::error::Error>> {
    if Command::new("yt-dlp").arg("--version").output().is_err() {
        eprintln!("Error: yt-dlp is not installed. Please install it to continue.");
        std::process::exit(1);
    }
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        eprintln!("Error: ffmpeg is not installed. Please install it to continue.");
        std::process::exit(1);
    }
    let args = Args::parse();
    let (client_id,client_secret)=check_configuration();
    // TODO: Create one client, and reference it in authentication so I dont create duplicates
    let auth_token=get_authentication_token(&client_id, &client_secret).await?;
    let playlist_url=format!("https://api.spotify.com/v1/playlists/{}/items",args.playlist);
    let client=reqwest::Client::new();
    let body = client.get(playlist_url).bearer_auth(&auth_token).send().await?;
    let json: serde_json::Value=body.json().await?;
    // TODO: I believe I can optimise this with structs later. But temporarily this will work
    let mut song_list: Vec<(String,String)>=Vec::new();
    // let mut count=0;
    if let Some(items)=json["items"].as_array(){
        for item in items {
            let mut artists=String::new();
            if let Some(artist_list) = item["item"]["artists"].as_array(){
                for (index,artist) in artist_list.iter().enumerate() {
                        let artist_name=artist["name"].as_str().unwrap_or("err");
                        artists.push_str(artist_name);
                        if index<artist_list.len()-1{
                            artists.push_str(", ");
                        }
                }
            }
            let name=item["item"]["name"].as_str().unwrap_or("invalid string").to_string();
            // println!("Artists: {}, Name: {}, count: {}", &artists, &name, &count);
            // count+=1;
            song_list.push((artists, name));
        }
    }
    let mut file=File::create("songs.txt")?;
    for (artists, song_name) in &song_list{
        writeln!(file, "ytsearch1:{} - {} Topic", artists, song_name)?;
    }
    let _download_songs=match download_songs(){
        std::result::Result::Ok(_)=>(),
        std::result::Result::Err(err)=>{
            println!("error downloading songs: {}", err); 
        }
    };
    fs::remove_file("songs.txt")?; // remove file at the end
    Ok(())
}
