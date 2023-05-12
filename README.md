# OpenGraph Fetcher

Fetches [OpenGraph](https://opengraphprotocol.org) metadata from arbitrary http urls.

## Usage Example

```bash
cargo run
```

```bash
$ curl -s 'localhost:8000/open.spotify.com/show/7HinkS0WZqDuMXYh02EUY1' | jq .
{
  "og:image": "https://i.scdn.co/image/ab6765630000ba8ad8d01f6463018f1644db243c",
  "og:site_name": "Spotify",
  "og:title": "The Blindboy Podcast",
  "og:url": "https://open.spotify.com/show/7HinkS0WZqDuMXYh02EUY1",
  "og:type": "website",
  "og:restrictions:country:allowed": "ZW",
  "og:description": "Listen to The Blindboy Podcast on Spotify. Hosted by Blindboy, of the Rubberbandits. An eclectic podcast containing short fiction, interviews and comedy. Hosted on Acast. See acast.com/privacy for more information."
}
```