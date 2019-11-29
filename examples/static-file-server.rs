use tide_static_files::{serve_static_files, StaticDirServer};

#[async_std::main]
async fn main() {
    let mut app = tide::with_state(StaticDirServer::new("./examples/"));
    app.at("/*").get(serve_static_files);
    app.listen("127.0.0.1:8000").await.unwrap();
}
