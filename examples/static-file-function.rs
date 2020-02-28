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
async fn main() -> std::io::Result<()> {
    let mut app = tide::with_state(AppState {
        static_root_dir: "./examples/static-example-files/".into(),
    });

    app.at("static/*path")
        .get(|req| async { serve_static_files(req).await.unwrap() });

    app.listen("127.0.0.1:8000").await
}
