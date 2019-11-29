//! Code heavily based on https://github.com/http-rs/tide/blob/4aec5fe2bb6b8202f7ae48e416eeb37345cf029f/backup/examples/staticfile.rs

use http::{header, StatusCode};
use tide::{Request, Response};

use async_std::{fs, io};
use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;

const DEFAULT_4XX_BODY: &str = "Oops! I can't find what you're looking for...";
const DEFAULT_5XX_BODY: &str = "I'm broken, apparently.";

/// Simple static file handler for Tide inspired from https://github.com/iron/staticfile.
#[derive(Clone)]
pub struct StaticDirServer {
    root: PathBuf,
}

impl StaticDirServer {
    /// Creates a new instance of this handler.
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = PathBuf::from(root.as_ref());
        if !root.exists() {
            // warn maybe?
        }

        StaticDirServer { root }
    }

    fn stream_bytes<'fut>(
        &'fut self,
        actual_path: String,
    ) -> Pin<Box<dyn Future<Output = io::Result<Response>> + 'fut + Send + Sync>> {
        Box::pin(async move {
            let path = &self.get_path(&actual_path);
            let meta = fs::metadata(path).await.ok();

            // If the file doesn't exist, then bail out.
            let meta = match meta {
                Some(m) => m,
                None => {
                    return Ok(tide::Response::new(StatusCode::NOT_FOUND.as_u16())
                        .set_header(header::CONTENT_TYPE.as_str(), mime::TEXT_HTML.as_ref())
                        .body_string(DEFAULT_4XX_BODY.into()));
                }
            };

            // Handle if it's a directory containing `index.html`
            if meta.is_dir() {
                // Redirect if path is a dir and URL doesn't end with "/"
                if !actual_path.ends_with("/") {
                    return Ok(tide::Response::new(StatusCode::MOVED_PERMANENTLY.as_u16())
                        .set_header(header::LOCATION.as_str(), String::from(actual_path) + "/")
                        .body_string("".into()));
                } else {
                    let index = Path::new(&actual_path)
                        .join("index.html")
                        .to_string_lossy()
                        .into_owned();
                    return self.stream_bytes(index).await;
                }
            }

            // We're done with the checks. Stream file!
            self.stream_file(path, meta).await
        })
    }

    async fn stream_file(
        &self,
        path: impl AsRef<Path>,
        meta: fs::Metadata,
    ) -> io::Result<Response> {
        let path = path.as_ref();
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let size = format!("{}", meta.len());

        let file = fs::File::open(PathBuf::from(path)).await?;
        let reader = io::BufReader::new(file);
        Ok(tide::Response::new(StatusCode::OK.as_u16())
            .body(reader)
            .set_header(header::CONTENT_LENGTH.as_str(), size)
            .set_mime(mime))
    }

    /// Percent-decode, normalize path components and return the final path joined with root.
    /// See https://github.com/iron/staticfile/blob/master/src/requested_path.rs
    fn get_path(&self, path: &str) -> PathBuf {
        let rel_path = Path::new(path)
            .components()
            .fold(PathBuf::new(), |mut result, p| {
                match p {
                    Component::Normal(x) => result.push({
                        let s = x.to_str().unwrap_or("");
                        percent_encoding::percent_decode(s.as_bytes())
                            .decode_utf8_lossy()
                            .as_ref()
                    }),
                    Component::ParentDir => {
                        result.pop();
                    }
                    _ => (), // ignore any other component
                }

                result
            });
        self.root.join(rel_path)
    }
}

pub async fn serve_static_files(ctx: Request<StaticDirServer>) -> Response {
    let path = ctx.uri().path();
    let resp = ctx.state().stream_bytes(path.into()).await;
    resp.map_err(|_| {
        tide::Response::new(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
            .set_header(header::CONTENT_TYPE.as_str(), mime::TEXT_HTML.as_ref())
            .body_string(DEFAULT_5XX_BODY.into())
    })
    .unwrap()
}
