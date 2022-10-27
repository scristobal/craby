use super::base;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub const MODEL_VERSION: &str = "2af375da21c5b824a84e1c459f45b69a117ec8649c2aa974112d7cf1840fc0ce";

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub text: String,
    pub seed: Option<u32>,
    pub grid_size: Option<u32>,
}

pub type Output = Option<Vec<String>>;

pub type Request = base::Request<Input>;
pub type Response = base::Response<Input, Output>;
