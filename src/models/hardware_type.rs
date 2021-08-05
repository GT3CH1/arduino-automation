use std::str::FromStr;
use serde::{Serialize, Deserialize};

/// Represents hardware types
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    ARDUINO,
    PI,
    OTHER,
    LG,
}

impl FromStr for Type {
    type Err = ();
    fn from_str(s: &str) -> Result<Type, ()> {
        match s {
            "ARDUINO" => Ok(Type::ARDUINO),
            "PI" => Ok(Type::PI),
            "OTHER" => Ok(Type::OTHER),
            "LG" => Ok(Type::LG),
            _ => Err(())
        }
    }
}
