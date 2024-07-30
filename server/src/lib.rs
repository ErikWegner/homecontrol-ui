use crate::mqtta::mqtta;

mod mqtta;

pub async fn run() {
    mqtta().await;
}
