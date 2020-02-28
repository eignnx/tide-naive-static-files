use async_std::task;
use tide_naive_static_files::StaticFilesEndpoint;

fn main() {
    let mut app = tide::new();
    app.at("/static").strip_prefix().get(StaticFilesEndpoint {
        root: "./examples/static-example-files/".into(),
    });

    task::block_on(async move { app.listen("127.0.0.1:8000").await.unwrap() });
}
