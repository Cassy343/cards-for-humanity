use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Pack {
    pub name: String,
    pub official: bool,
    #[serde(rename = "black")]
    pub responses: Vec<Response>,
    #[serde(rename = "white")]
    pub prompts: Vec<Prompt>,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Prompt {
    pub text: String,
    pub pick: u8,
}

pub type Response = String;
