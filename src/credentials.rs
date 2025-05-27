use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String
}