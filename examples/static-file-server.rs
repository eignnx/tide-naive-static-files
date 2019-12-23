use async_std::task;
use tide_naive_static_files::StaticFilesEndpoint;

struct AppState {}

fn main() {
    let state = AppState {};

    let mut app = tide::with_state(state);
    app.at("/static").strip_prefix().get(StaticFilesEndpoint {
        root: "./examples/".into(),
    });

    task::block_on(async move { app.listen("127.0.0.1:8000").await.unwrap() });
}
