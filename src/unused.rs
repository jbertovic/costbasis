#[derive(Debug, PartialEq)]
pub enum GainType {
    Shortterm,
    Longterm,
}

impl std::str::FromStr for GainType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Short" | "Shortterm" | "st" | "short" => Ok(GainType::Shortterm),
            "Long" | "Longterm" | "lt" | "long" => Ok(GainType::Longterm),
            _ => Err(format!("'{}' is not a valid value for GainType", s)),
        }
    }
