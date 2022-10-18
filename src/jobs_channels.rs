pub struct JobRequest {
    pub prompt: String,
    pub channel_id: String,
}

pub struct JobResult {
    pub url: Option<String>,
    pub error: Option<String>,
    pub channel_id: String,
}
