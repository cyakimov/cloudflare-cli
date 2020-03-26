#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum GlobalUser {
    TokenAuth { api_token: String },
    GlobalKeyAuth { email: String, api_key: String },
}
