use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    ARDUINO,
    PI,
    OTHER,
}

impl FromStr for Type {
    type Err = ();
    fn from_str(s: &str) -> Result<Type, ()> {
        match s {
            "ARDUINO" => Ok(Type::ARDUINO),
            "PI" => Ok(Type::PI),
            "OTHER" => Ok(Type::OTHER),
            _ => Err(())
        }
    }
}
