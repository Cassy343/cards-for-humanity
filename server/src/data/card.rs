use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CardID {
    pub pack_number: usize,
    pub card_number: usize,
}

impl CardID {
    pub fn new(pack_number: usize, card_number: usize) -> Self {
        CardID {
            pack_number,
            card_number,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pack {
    pub name: String,
    pub official: bool,
    #[serde(
        rename = "white",
        serialize_with = "Pack::serialize_responses",
        deserialize_with = "Pack::deserialize_responses"
    )]
    pub responses: Vec<Response>,
    #[serde(rename = "black")]
    pub prompts: Vec<Prompt>,
}

impl Pack {
    pub fn meta(&self) -> PackMeta {
        PackMeta {
            official: self.official,
            num_prompts: self.prompts.len(),
            num_responses: self.responses.len(),
        }
    }

    fn serialize_responses<S>(responses: &[Response], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(responses.len()))?;
        for response in responses {
            seq.serialize_element(&RawResponse::from(&**response))?;
        }
        seq.end()
    }

    fn deserialize_responses<'de, D>(deserializer: D) -> Result<Vec<Response>, D::Error>
    where D: Deserializer<'de> {
        let responses: Vec<&'de str> = Deserialize::deserialize(deserializer)?;
        Ok(responses.into_iter().map(Into::into).collect())
    }
}

impl PartialEq for Pack {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Pack {}

#[derive(Clone, Copy)]
pub struct PackMeta {
    pub official: bool,
    pub num_prompts: usize,
    pub num_responses: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Prompt {
    pub text: String,
    pub pick: usize,
}

pub type Response = String;

// This technically means we're missing the `pack` field on any custom cards serialized, this shouldn't matter tho cause we never use it
#[derive(Deserialize, Serialize)]
struct RawResponse<'a> {
    text: &'a str,
}

impl<'a> From<&'a str> for RawResponse<'a> {
    fn from(text: &'a str) -> Self {
        Self { text }
    }
}

impl From<RawResponse<'_>> for Response {
    fn from(response: RawResponse<'_>) -> Self {
        response.text.into()
    }
}
