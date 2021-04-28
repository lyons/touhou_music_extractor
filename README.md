# Touhou Music Extractor

I needed to extract unlooped background music from a few Touhou games, did not care to trust the binary downloads of the [existing tools](https://en.touhouwiki.net/wiki/Game_Tools_and_Modifications#Extractors) for this, and didn't feel like trying to build any of them from source -- so I wrote my own tool to do it instead.

`cargo build --release` to build executable `thme`

`thme help` or `cargo run --release -- help` for usage instructions.

Supports extracting music from games that use PCM WAV data (all mainline shooting games), but not from games that use OGG data (fighting games). The tool can extract one or all tracks from a given game, can loop the track a fixed number of times or for a fixed duration, and can fade out the end of the track for a specified time.
