use tide_naive_static_files::StaticFilesEndpoint;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut app = tide::new();
    app.at("/static").strip_prefix().get(StaticFilesEndpoint {
        root: "./examples/static-example-files/".into(),
    });

    app.listen("127.0.0.1:8000").await
}
