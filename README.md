# Compressor

A simple audio compression programme, using techniques similar to MP3.

Currently targets Linux only.

This is a proof of concept. Do not use in production, do not expect support, do not submit PRs, issue tickets.

## Installation

Requires [Rust](https://rustup.rs/).

## Usage

`compressor` only accepts wav files.

Encode a wav file:

```bash
# .xprs is the suggested files extension for compressed audio files
cargo run --release encode audio/mywavfile.wav audio/mycompressedfile.xprs
```

Decode and play a compressed file:

```bash
cargo run --release decode audio/mycompressedfile.xprs
```

