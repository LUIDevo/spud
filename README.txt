# Spud - A Spotify Downloader Alternative
*Spud* is a playlist downloader built in Rust. It uses the Spotify API to fetch playlist metadata, and retrieves audio using yt-dlp to get mp3 files.

I built Spud as a simple, reliable alternative to SpotDL which I noted has become more error prone as of 2026, and I wanted to make an alternative for myself that feels more stable.

# Prerequisites
yt-dlp
ffmpeg
Spotify Web API ID and Secret
Since this app is in early development, it requires the rust toolchain to run.

To obtain a Spotify Web API ID and Secret,
1. Go to developer.spotify.com/dashboard
2. Create a new app 
3. *CRITICAL*: For this app, you must set the *Redirect URI* to http://127.0.0.1:8888/callback`
4. Get your credentials (*Client ID* and *Client Secret*)
5. Paste in credentials when prompted (will be saved locally)

# Usage
Spud requires the playlist id.

A playlist id looks like:
`https://open.spotify.com/playlist/PLAYLIST_ID`
Or
`https://open.spotify.com/playlist/PLAYLIST_ID?si=...`

`--playlist [PLAYLIST_ID]`

> Does this work without Premium?
This app uses the Spotify Web API, which unfortunately requires a premium account as of Febuary 11 2026.

> Special Mentions
yt-dlp is used to download songs from youtube

# Disclaimer:
This tool is for personal use only. Please support the artists by listening to their music on official platforms.

# TODO:
- [ ] CLI
- [ ] Add logging
- [ ] Add progress bars
- [ ] Add Windows Support
- [ ] Add better error handling for incorrect credentials
