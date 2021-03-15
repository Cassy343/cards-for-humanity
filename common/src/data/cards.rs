use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

#[derive(Serialize, Deserialize)]
pub struct Pack {
    pub name: String,
    pub official: bool,
    #[serde(rename = "white")]
    #[serde(
        deserialize_with = "deserialize_response",
        serialize_with = "serialize_response"
    )]
    pub responses: Vec<Response>,
    #[serde(rename = "black")]
    pub prompts: Vec<Prompt>,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Prompt {
    pub text: String,
    pub pick: u8,
}

pub type Response = String;


// This technically means we're missing the `pack` field on any custom cards serialized, this shouldn't matter tho cause we never use it
#[derive(Deserialize, Serialize)]
struct RawResponse {
    text: String,
}
struct ResponseVisitor;

impl<'de> Visitor<'de> for ResponseVisitor {
    type Value = Vec<RawResponse>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a RawResponse object")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de> {
        let mut vec = Vec::new();

        while let Some(e) = seq.next_element()? {
            vec.push(e)
        }

        Ok(vec)
    }
}

fn deserialize_response<'de, D>(d: D) -> Result<Vec<Response>, D::Error>
where D: Deserializer<'de> {
    let v = d.deserialize_seq(ResponseVisitor)?;
    Ok(v.iter().map(|r| r.text.to_owned()).collect())
}

fn serialize_response<S>(responses: &Vec<Response>, s: S) -> Result<S::Ok, S::Error>
where S: Serializer {
    let mut seq = s.serialize_seq(Some(responses.len()))?;

    for response in responses {
        seq.serialize_element(&RawResponse {
            text: response.to_owned(),
        })?;
    }

    seq.end()
}
