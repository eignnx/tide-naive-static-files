# tide-naive-static-files
A simple static file serving component for Rust's Tide web framework.

## Acknowledgements 
This code is based heavily on [this archived example](https://github.com/http-rs/tide/blob/4aec5fe2bb6b8202f7ae48e416eeb37345cf029f/backup/examples/staticfile.rs).

## Example
```rust
use tide_naive_static_files::{serve_static_files, StaticDirServer};

#[async_std::main]
async fn main() {
    let mut app = tide::with_state(StaticDirServer::new("./public/").unwrap());
    app.at("/*")
        .get(|req| async { serve_static_files(req).await.unwrap() });
    app.listen("127.0.0.1:8000").await.unwrap();
}
```

## Problems
Right now it does not use `AsyncBufRead` when putting data into the `http::Response`. This means it loads the file data into memory before sending it, so if you need to send gigantic files, that could be a problem. If you know of a solution, please open an issue on the repository!