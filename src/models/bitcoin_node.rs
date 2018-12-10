use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinNode {
    pub url: String,
    pub user: String,
    pub password: String,
    pub quarantine: Quarantine,
}

impl BitcoinNode {
    pub fn new(url: String, user: String, password: String) -> Self {
        Self {
            url,
            user,
            password,
            quarantine: Quarantine::No,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Quarantine {
    No,
    Yes(NaiveDateTime),
}
