use std::path::{Path, PathBuf};
use tide_naive_static_files::{serve_static_files, StaticRootDir};

struct AppState {
    static_root_dir: PathBuf,
}

impl StaticRootDir for AppState {
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
    app.at("/*")
        .get(|req| async { serve_static_files(req).await.unwrap() });
    app.listen("127.0.0.1:8000").await.unwrap();
}
