//! Code heavily based on https://github.com/http-rs/tide/blob/4aec5fe2bb6b8202f7ae48e416eeb37345cf029f/backup/examples/staticfile.rs

use http::{
    header::{self, HeaderMap},
    StatusCode,
};
use tide::{Endpoint, Request, Response, Result};

use async_std::{fs, io, task};
use std::path::{Component, Path, PathBuf};
use async_std::future;
use async_std::task::{Context, Poll};

use std::pin::Pin;

pub trait StaticRootDir {
    fn root_dir(&self) -> &Path;
}

impl<T: StaticRootDir> StaticRootDir for &T {
    fn root_dir(&self) -> &Path {
        (*self).root_dir()
    }
}

fn stream_bytes(
    root: &Path,
    actual_path: &str,
    headers: &HeaderMap,
) -> io::Result<Response> {
    let path = &get_path(&root, actual_path);
    let meta = task::block_on(fs::metadata(path)).ok();

    // If the file doesn't exist, then bail out.
    let meta = match meta {
        Some(m) => m,
        None => {
            return Ok(tide::Response::new(StatusCode::NOT_FOUND.as_u16())
                .set_header(header::CONTENT_TYPE.as_str(), mime::TEXT_HTML.as_ref())
                .body_string(format!("Couldn't locate file {:?}", actual_path)));
        }
    };

    // Handle if it's a directory containing `index.html`
    if !meta.is_file() {
        // Redirect if path is a dir and URL doesn't end with "/"
        if !actual_path.ends_with("/") {
            return Ok(tide::Response::new(StatusCode::MOVED_PERMANENTLY.as_u16())
                .set_header(header::LOCATION.as_str(), String::from(actual_path) + "/")
                .body_string("".into()));
        } else {
            let index = Path::new(actual_path).join("index.html");
            return stream_bytes(root, &*index.to_string_lossy(), headers);
        }
    }

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let size = format!("{}", meta.len());

    // We're done with the checks. Stream file!
    let file = task::block_on(fs::File::open(PathBuf::from(path))).unwrap();
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

pub struct StaticFilesEndpoint {
    pub root: PathBuf,
}

impl<State> Endpoint<State> for StaticFilesEndpoint {
    type Fut = Future;

    fn call(&self, ctx: Request<State>) -> Self::Fut {
        let path = ctx.uri().path().to_string();

        let resp = stream_bytes(&self.root, &path, ctx.headers());
        match resp {
            Err(e) => {
                eprintln!("tide-naive-static-files internal error: {}", e);
                let resp = tide::Response::new(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
                    .set_header(header::CONTENT_TYPE.as_str(), mime::TEXT_HTML.as_ref())
                    .body_string("Internal server error!".into());
                Future { res: Some(resp) }
            }
            Ok(resp) => Future { res: Some(resp) }
        }
    }
}

/// Future returned from `redirect`.
pub struct Future {
    res: Option<Response>,
}

impl future::Future for Future {
    type Output = Response;
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.res.take().unwrap())
    }
}
