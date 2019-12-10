//! Code heavily based on https://github.com/http-rs/tide/blob/4aec5fe2bb6b8202f7ae48e416eeb37345cf029f/backup/examples/staticfile.rs

use http::{
    header::{self, HeaderMap},
    StatusCode,
};
use tide::{Request, Response, Result};

use async_std::{fs, io, task};
use std::path::{Component, Path, PathBuf};

const DEFAULT_5XX_BODY: &str = "I'm broken, apparently.";

pub trait StaticRootDir {
    fn root_dir(&self) -> &Path;
}

impl<T: StaticRootDir> StaticRootDir for &T {
    fn root_dir(&self) -> &Path {
        (*self).root_dir()
    }
}

fn stream_bytes(
    root: impl StaticRootDir,
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
fn get_path(root: impl StaticRootDir, path: &str) -> PathBuf {
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
    root.root_dir().join(rel_path)
}

pub async fn serve_static_files(ctx: Request<impl StaticRootDir>) -> Result {
    let path: String = ctx.param("path").expect(
        "`tide_naive_static_files::serve_static_files` requires a `*path` glob param at the end!",
    );
    let root = ctx.state();
    let resp = stream_bytes(root, &path, ctx.headers());
    match resp {
        Err(_) => {
            let resp = tide::Response::new(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
                .set_header(header::CONTENT_TYPE.as_str(), mime::TEXT_HTML.as_ref())
                .body_string(DEFAULT_5XX_BODY.into());
            Ok(resp)
        }
        Ok(resp) => Ok(resp),
    }
}
