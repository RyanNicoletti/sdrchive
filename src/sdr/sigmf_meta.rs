use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SigmfMeta {
    pub global: GlobalSection,
}

#[derive(Deserialize, Debug)]
pub struct GlobalSection {
    #[serde(rename = "core:sample_rate")]
    pub sample_rate: u64,
    #[serde(rename = "core:datatype")]
    pub datatype: String,
}
