use tokio::sync::Mutex;

pub struct Requests {
    pub counter: Mutex<i32>,
}

impl Requests {
    pub async fn increment(&self) {
        let mut counter = self.counter.lock().await;
        *counter += 1;
    }

    pub async fn read(&self) -> i32 {
        *self.counter.lock().await
    }
}
