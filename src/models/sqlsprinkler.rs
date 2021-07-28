use serde::Deserialize;

#[derive(Deserialize)]
pub struct SystemState {
    pub system_enabled: bool,
}
