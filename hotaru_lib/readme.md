# hotaru_lib

[![Crates.io](https://img.shields.io/crates/v/hotaru_lib)](https://crates.io/crates/hotaru_lib)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Utility library for the Hotaru web framework, providing common functionality for URL encoding, compression, encryption, and random string generation.

## Features

- **url_encoding** - URL percent-encoding utilities
- **compression** - Support for gzip, brotli, and zstd compression
- **ende** - Encryption/decryption using AES-GCM and PBKDF2
- **random** - Secure random string generation

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
hotaru_lib = { version = "0.7.3", features = ["url_encoding", "compression"] }
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `url_encoding` | URL percent-encoding | ✓ |
| `random` | Random string generation | ✓ |
| `compression` | gzip, brotli, zstd support | |
| `ende` | Encryption/decryption | |

## License

MIT License

## Part of Hotaru Framework

This is a utility crate for the [Hotaru web framework](https://crates.io/crates/hotaru).

Learn more: https://fds.rs
