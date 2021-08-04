use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    LIGHT,
    SWITCH,
    GARAGE,
    SPRINKLER,
    ROUTER,
    SqlSprinklerHost,
    TV,
}

impl FromStr for Type {
    type Err = ();
    fn from_str(s: &str) -> Result<Type, ()> {
        match s {
            "LIGHT" => Ok(Type::LIGHT),
            "SWITCH" => Ok(Type::SWITCH),
            "GARAGE" => Ok(Type::GARAGE),
            "SPRINKLER" => Ok(Type::SPRINKLER),
            "ROUTER" => Ok(Type::ROUTER),
            "SQLSPRINKLER_HOST" => Ok(Type::SqlSprinklerHost),
            "TV" => Ok(Type::TV),
            _ => Err(())
        }
    }
}