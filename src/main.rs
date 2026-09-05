use head_of_site::router;

#[tokio::main]
async fn main() {
    topcoat::start(router()).await.unwrap();
}
