use std::collections::HashMap;
use serde_derive::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Adapter {
    #[serde(skip, default = "String::new")]
    pub name:     String,
    pub inputs:   Vec<Device>,
    pub outputs:  Vec<Output>,
    pub autoplug: Vec<(String, String)>,
    pub map:      HashMap<(String, String), Vec<Mapper>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Device {
    pub name:    String,
    pub r#match: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Output {
    pub name:    String,
    pub r#match: String,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Mapper {
    #[serde(alias = "Button")]
    Key(String, String),
    #[serde(alias = "ButtonReversed")]
    KeyReversed(String, String),
    #[serde(alias = "ButtonToAxis")]
    KeyToAxis(String, String),
    Axis(String, String),
    AxisReversed(String, String),
    #[serde(alias = "AxisToButtons")]
    AxisToKeys(String, String, String, isize),
    #[serde(alias = "HatToButtons")]
    HatToKeys(String, String, String),
    Motion(String, String),
}
