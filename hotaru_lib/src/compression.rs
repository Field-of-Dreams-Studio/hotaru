//! HTTP content compression and decompression utilities
//!
//! This module provides functions for handling common HTTP content encodings
//! according to RFC 9110. It supports both compression and decompression
//! for standard algorithms used in HTTP transfers.
//!
//! # Supported Algorithms
//!
//! | Algorithm    | Encoding Name | Functions                     |
//! |--------------|---------------|-------------------------------|
//! | GZIP         | `gzip`        | `compress_gzip`, `decompress_gzip` |
//! | DEFLATE      | `deflate`     | `compress_deflate`, `decompress_deflate` |
//! | Brotli       | `br`          | `compress_brotli`, `decompress_brotli` |
//! | Zstandard    | `zstd`        | `compress_zstd`, `decompress_zstd` |
//!
//! # Examples
//!
//! ## Decompression
//! ```
//! use hotaru::http::compression::decompress_gzip;
//!
//! let compressed_data = [0x1f, 0x8b, 0x08, 0x00, ...];
//! let decompressed = decompress_gzip(&compressed_data).unwrap();
//! assert_eq!(decompressed, b"Hello world!");
//! ```
//!
//! ## Compression
//! ```
//! use hotaru::http::compression::compress_gzip;
//!
//! let data = b"Hello world!";
//! let compressed = compress_gzip(data).unwrap();
//! // Send compressed data with Content-Encoding: gzip
//! ```

use flate2::{bufread, write, Compression};
use brotli::{CompressorWriter as BrotliCompressor, Decompressor as BrotliDecompressor};
use zstd::stream::{read::Decoder as ZstdDecoder, write::Encoder as ZstdEncoder};
use std::io::{Read, Write}; 

static CHUNK_SIZE: usize = 4096; 

/// Decompresses GZIP-encoded data
///
/// # Arguments
///
/// * `data` - GZIP-compressed byte slice
///
/// # Returns
///
/// Decompressed data as `Vec<u8>` or `std::io::Error` on failure
///
/// # Example
/// ```
/// # use hotaru::http::compression::decompress_gzip;
/// let data = [0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 
///             0xf3, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0x08, 0xcf, 0x2f, 0xca, 
///             0x49, 0x51, 0x04, 0x00, 0x00, 0x00, 0xff, 0xff, 0x03, 0x00, 
///             0x0c, 0x6e, 0x81, 0xec, 0x0c, 0x00, 0x00, 0x00];
/// let decompressed = decompress_gzip(&data).unwrap();
/// assert_eq!(decompressed, b"Hello world!");
/// ```
pub fn decompress_gzip(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = bufread::GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Compresses data using GZIP encoding
///
/// # Arguments
///
/// * `data` - Raw byte slice to compress
///
/// # Returns
///
/// GZIP-compressed data as `Vec<u8>` or `std::io::Error` on failure
///
/// # Example
/// ```
/// # use hotaru::http::compression::compress_gzip;
/// let data = b"Hello world!";
/// let compressed = compress_gzip(data).unwrap();
/// ```
pub fn compress_gzip(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = write::GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// Decompresses DEFLATE-encoded data
///
/// # Arguments
///
/// * `data` - DEFLATE-compressed byte slice
///
/// # Returns
///
/// Decompressed data as `Vec<u8>` or `std::io::Error` on failure
///
/// # Example
/// ```
/// # use hotaru::http::compression::decompress_deflate;
/// let data = [0xf3, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0x08, 0xcf, 0x2f, 0xca, 
///             0x49, 0x51, 0x04, 0x00];
/// let decompressed = decompress_deflate(&data).unwrap();
/// assert_eq!(decompressed, b"Hello world!");
/// ```
pub fn decompress_deflate(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = bufread::DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Compresses data using DEFLATE encoding
///
/// # Arguments
///
/// * `data` - Raw byte slice to compress
///
/// # Returns
///
/// DEFLATE-compressed data as `Vec<u8>` or `std::io::Error` on failure
pub fn compress_deflate(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = write::DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// Decompresses Brotli-encoded data
///
/// # Arguments
///
/// * `data` - Brotli-compressed byte slice
///
/// # Returns
///
/// Decompressed data as `Vec<u8>` or `std::io::Error` on failure
///
/// # Example
/// ```
/// # use hotaru::http::compression::decompress_brotli;
/// let data = [0x0b, 0x00, 0x80, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 
///             0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x03];
/// let decompressed = decompress_brotli(&data).unwrap();
/// assert_eq!(decompressed, b"Hello world!");
/// ```
pub fn decompress_brotli(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    BrotliDecompressor::new(data, CHUNK_SIZE)
        .read_to_end(&mut decompressed)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(decompressed)
}

/// Compresses data using Brotli encoding
///
/// # Arguments
///
/// * `data` - Raw byte slice to compress
///
/// # Returns
///
/// Brotli-compressed data as `Vec<u8>` or `std::io::Error` on failure
pub fn compress_brotli(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut compressor = BrotliCompressor::new(Vec::new(), 4096, 11, 22);
    compressor.write_all(data)?;
    Ok(compressor.into_inner())
}

/// Decompresses Zstandard-encoded data
///
/// # Arguments
///
/// * `data` - Zstandard-compressed byte slice
///
/// # Returns
///
/// Decompressed data as `Vec<u8>` or `std::io::Error` on failure
///
/// # Example
/// ```
/// # use hotaru::http::compression::decompress_zstd;
/// let data = [0x28, 0xb5, 0x2f, 0xfd, 0x24, 0x00, 0x01, 0x00, 
///             0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 
///             0x6f, 0x72, 0x6c, 0x64, 0x21, 0x00, 0x00, 0x00];
/// let decompressed = decompress_zstd(&data).unwrap();
/// assert_eq!(decompressed, b"Hello world!");
/// ```
pub fn decompress_zstd(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = ZstdDecoder::new(data)?;
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Compresses data using Zstandard encoding
///
/// # Arguments
///
/// * `data` - Raw byte slice to compress
/// * `level` - Compression level (1-21, where 1 is fastest, 21 is best compression)
///
/// # Returns
///
/// Zstandard-compressed data as `Vec<u8>` or `std::io::Error` on failure
pub fn compress_zstd(data: &[u8], level: i32) -> std::io::Result<Vec<u8>> {
    let mut encoder = ZstdEncoder::new(Vec::new(), level)?;
    encoder.write_all(data)?;
    encoder.finish()
}
