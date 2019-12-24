//! Code heavily based on https://github.com/http-rs/tide/blob/4aec5fe2bb6b8202f7ae48e416eeb37345cf029f/backup/examples/staticfile.rs

use async_std::{fs, future, io};
use http::{
    header::{self},
    StatusCode,
};
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use tide::{Endpoint, Request, Response};

async fn stream_bytes(root: PathBuf, actual_path: &str) -> io::Result<Response> {
    let mut path = get_path(&root, actual_path);

    // Loop if the path points to a directory because we want to try looking for
    // an "index.html" file within that directory.
    let (meta, path): (fs::Metadata, PathBuf) = loop {
        let meta = fs::metadata(&path).await.ok();

        // If the file doesn't exist, then bail out.
        if meta.is_none() {
            return Ok(tide::Response::new(StatusCode::NOT_FOUND.as_u16())
                .set_header(header::CONTENT_TYPE.as_str(), mime::TEXT_HTML.as_ref())
                .body_string(format!("Couldn't locate requested file {:?}", actual_path)));
        }
        let meta = meta.unwrap();

        // If the path points to a directory, look for an "index.html" file.
        if !meta.is_file() {
            path.push("index.html");
            continue; // Try again.
        } else {
            break (meta, path);
        }
    };

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let size = format!("{}", meta.len());

    // We're done with the checks. Stream file!
    let file = fs::File::open(PathBuf::from(&path)).await.unwrap();
    let reader = io::BufReader::new(file);
    Ok(tide::Response::new(StatusCode::OK.as_u16())
        .body(reader)
        .set_header(header::CONTENT_LENGTH.as_str(), size)
        .set_mime(mime))
}

/// Percent-decode, normalize path components and return the final path joined with root.
/// See https://github.com/iron/staticfile/blob/master/src/requested_path.rs
fn get_path(root: &Path, path: &str) -> PathBuf {
    let rel_path = Path::new(path)
        .components()
        .fold(PathBuf::new(), |mut result, p| {
            match p {
                Component::Normal(x) => result.push({
                    let s = x.to_str().unwrap_or("");
                    &*percent_encoding::percent_decode(s.as_bytes()).decode_utf8_lossy()
                }),
                Component::ParentDir => {
                    result.pop();
                }
                _ => (), // ignore any other component
            }

            result
        });
    root.join(rel_path)
}

type BoxFuture<T> = Pin<Box<dyn future::Future<Output = T> + Send>>;

pub struct StaticFilesEndpoint {
    pub root: PathBuf,
}

impl<State> Endpoint<State> for StaticFilesEndpoint {
    type Fut = BoxFuture<Response>;

    fn call(&self, ctx: Request<State>) -> Self::Fut {
        let path = ctx.uri().to_string();
        let root = self.root.clone();

        Box::pin(async move {
            stream_bytes(root, &path).await.unwrap_or_else(|e| {
                eprintln!("tide-naive-static-files internal error: {}", e);
                internal_server_error("Internal server error reading file")
            })
        })
    }
}

fn internal_server_error(body: &'static str) -> Response {
    tide::Response::new(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
        .set_header(header::CONTENT_TYPE.as_str(), mime::TEXT_HTML.as_ref())
        .body_string(body.into())
}
