use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ImportSettings {
    #[serde(default)]
    pub filename: Option<String>,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self { filename: None }
    }
}
