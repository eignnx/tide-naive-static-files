# tide-naive-static-files

A simple static file serving component for Rust's Tide web framework.

## Acknowledgements

This code is based heavily on [this archived example](https://github.com/http-rs/tide/blob/4aec5fe2bb6b8202f7ae48e416eeb37345cf029f/backup/examples/staticfile.rs).

## Example

To use the library:

1. Define some state for your server.
2. Implement `StaticRootDir` on your state. This tells the library how to access the name of the root directory in which your static assets live.
3. Set up a `get` endpoint with a `*path` glob pattern (like `/static/*path` or `/*path`) and have it call the `serve_static_files` function.

```rust
use std::path::{Path, PathBuf};
use tide_naive_static_files::{serve_static_files, StaticRootDir};

struct AppState { // 1.
    static_root_dir: PathBuf,
}

impl StaticRootDir for AppState { // 2.
    fn root_dir(&self) -> &Path {
        &self.static_root_dir
    }
}

#[async_std::main]
async fn main() {
    let state = AppState {
        static_root_dir: "./examples/".into(),
    };

    let mut app = tide::with_state(state);
    app.at("/static/*path") // 3.
        .get(|req| async { serve_static_files(req).await.unwrap() });
    app.listen("127.0.0.1:8000").await.unwrap();
}
```

## Problems

Right now it kinda doesn't use all the async-y-ness that it probably could. There are a couple of unfortunate `task::block_on`s that I want to get rid of. Suggestions welcome!
