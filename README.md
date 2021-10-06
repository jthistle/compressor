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

## Details

### Encoding

The input file is divided up into 'chunks', with a set number of samples in each.

The Fourier transform of each source chunk is taken, and the N 'loudest' frequencies are kept,
the rest are discarded. These N frequencies and their Fourier transform values are written to
a file, in groups of N samples ('encoded chunks').

### Decoding

The inverse Fourier transform of each encoded chunk is taken, setting all frequencies not present in the file
to have a value of 0. This gives, more or less, the original signal.

### The file format

All numbers are 32-bit LE signed integers unless stated otherwise.

- 4 bytes: "XPRS" - the filetype magic number
- 4 bytes: the size of each source chunk
- 4 bytes: the number of chunks in the "DATA" section
- 4 bytes: the size of each encoded chunk
- 4 bytes: the (integer) sample frequency
- 4 bytes: the number of channels
- 4 bytes: "DATA" - the data header
- variable size: the compressed data chunks (interleaved), with each chunk in the format:
  - 2 bytes: the chunk discrete frequency (16-bit unsigned LE integer)
  - 4 bytes: the DFT real part (32-bit IEEE float, LE)
  - 4 bytes: the DFT imaginary part (32-bit IEEE float, LE)
