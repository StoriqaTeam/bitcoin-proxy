#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpsGenieResponse {
    pub result: String,
    pub took: f64,
    pub request_id: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpsGeniePayload {
    pub message: String,
    pub responders: Vec<OpsGenieUser>,
    pub visible_to: Vec<OpsGenieUser>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpsGenieUser {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

impl OpsGeniePayload {
    pub fn new(message: String, team_name: String) -> Self {
        Self {
            message,
            responders: vec![OpsGenieUser::new(team_name.clone())],
            visible_to: vec![OpsGenieUser::new(team_name.clone())],
        }
    }
}

impl OpsGenieUser {
    pub fn new(name: String) -> Self {
        Self {
            name,
            type_: "team".to_string(),
        }
    }
}
