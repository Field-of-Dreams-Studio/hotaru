use std::fmt::Write;

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};

use crate::connection::error::ConnectionError; 

use super::meta::HttpMeta; 
use super::body::HttpBody; 
use super::safety::HttpSafety; 

pub async fn parse_lazy<R: AsyncRead + Unpin>(
    stream: &mut BufReader<R>, 
    config: &HttpSafety, 
    is_request: bool, 
    print_raw: bool
) -> Result<(HttpMeta, HttpBody), ConnectionError> { 

    // Create one BufReader up-front, pass this throughout.
    let mut meta = HttpMeta::from_stream(
        stream, 
        config, 
        print_raw, 
        is_request 
    ).await?; 

    let body = HttpBody::read_buffer(
        stream, 
        &mut meta, 
        config)
    .await?;

    Ok((meta, body)) 
} 

pub async fn send<W: AsyncWrite +  Unpin>(mut meta: HttpMeta, body: HttpBody, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    let mut headers = String::with_capacity(256); 

    // Add the values such as content length into header 
    let bin = body.into_static(&mut meta).await; 
    write!( 
        &mut headers,
        "{}", 
        meta.represent()
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    writer.write_all(headers.as_bytes()).await?;
    writer.write_all(&bin).await?; 

    // println!("{:?}, {:?}", headers, bin); 
    writer.flush().await?; 
    
    Ok(()) 
} 
