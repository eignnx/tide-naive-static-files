use async_std::task;
use tide_static_files::{serve_static_files, StaticDirServer};

fn main() {
    let mut app = tide::with_state(StaticDirServer::new("./examples/").unwrap());
    app.at("/*")
        .get(|req| async { serve_static_files(req).await.unwrap() });
    task::block_on(app.listen("127.0.0.1:8000")).unwrap();
}
