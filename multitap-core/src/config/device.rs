use std::collections::HashMap;
use serde_derive::Deserialize;
use crate::input::{KeyId, AxisId, MotionId};

#[derive(Clone, Debug, Deserialize)]
pub struct Device {
    #[serde(skip, default = "String::new")]
    pub name:      String,
    pub fullname:  String,
    #[serde(rename = "match")]
    pub r#matches: Vec<Match>,
    #[serde(default = "HashMap::new")]
    pub default_adapters:  HashMap<Vec<String>, String>,
    #[serde(alias = "buttons")]
    #[serde(default = "HashMap::new")]
    pub keys: HashMap<String, KeyId>,
    #[serde(default = "HashMap::new")]
    pub axes: HashMap<String, AxisId>,
    #[serde(default = "HashMap::new")]
    pub motions: HashMap<String, MotionId>,
    pub leds: Option<Leds>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Match {
    #[serde(default = "Vec::new")]
    pub driver:  Vec<String>,
    #[serde(default = "Vec::new")]
    pub vendor:  Vec<i32>,
    #[serde(default = "Vec::new")]
    pub product: Vec<i32>,
}

#[derive(Debug)]
pub struct DeviceSpec {
    pub vendor:  i32,
    pub product: i32,
    pub driver:  String 
}

#[derive(Clone, Debug, Deserialize)]
pub struct Leds {
    pub r#match: String,
}

impl Device {

    pub fn find(&self, spec: &DeviceSpec) -> bool {
        for r#match in &self.matches {
            if r#match.check(spec) {
                return true;
            }
        }
        return false;
    }

    pub fn find_adapter(&self, ports: &[String]) -> Option<String> {
        self.default_adapters.get(ports).cloned()
    }
}

impl Match {

    fn check_one<T: Eq>(refvalue: &T, values: &[T]) -> bool {
        values.is_empty() || values.iter().any(|value| value == refvalue)
    }

    fn check(&self, spec: &DeviceSpec) -> bool {
        Match::check_one(&spec.driver, &self.driver) &&
            Match::check_one(&spec.vendor, &self.vendor) &&
            Match::check_one(&spec.product, &self.product)
    }
}