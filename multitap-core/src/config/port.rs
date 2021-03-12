use std::collections::HashMap;
use serde_derive::Deserialize;
use crate::input::{KeyId, AxisId, MotionId};

#[derive(Clone, Debug, Deserialize)]
pub struct Port {
    #[serde(skip, default = "String::new")]
    pub name:       String,
    pub product_id: i32,
    pub version:    i32,
    pub fullname:   String,
    #[serde(default = "Port::unique_slot")]
    pub slots:      Vec<String>,

    #[serde(alias = "buttons")]
    #[serde(default = "HashMap::new")]
    pub keys: HashMap<String, KeyId>,
    #[serde(default = "HashMap::new")]
    pub axes: HashMap<String, Axis>,
    #[serde(default = "HashMap::new")]
    pub motions: HashMap<String, MotionId>,
}

impl Port {
    fn unique_slot() -> Vec<String> { vec!["unique".to_string()] }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Axis {
    pub id: AxisId,
    pub min:  i32,
    pub max:  i32,
    pub flat: i32,
}
