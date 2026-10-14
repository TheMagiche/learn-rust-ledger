//! Spending buckets. A closed set is an enum, not a free-form string.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::SpendError;

/// LEARN(enums): every expense is one of these variants and nothing else.
/// LEARN(derive): `Debug`, `Copy`, `Serialize`, and the rest are traits the compiler implements for us.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Food,
    Transport,
    Rent,
    Utilities,
    Fun,
    Health,
    Other,
}

impl Category {
    pub const ALL: [Category; 7] = [
        Category::Food,
        Category::Transport,
        Category::Rent,
        Category::Utilities,
        Category::Fun,
        Category::Health,
        Category::Other,
    ];
}

/// LEARN(display): printing a category yields the same lowercase word the API uses.
impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Category::Food => "food",
            Category::Transport => "transport",
            Category::Rent => "rent",
            Category::Utilities => "utilities",
            Category::Fun => "fun",
            Category::Health => "health",
            Category::Other => "other",
        };
        formatter.write_str(label)
    }
}

/// LEARN(fromstr): `FromStr` is the reverse of `Display`. `"Food"` and `"FOOD"` both work.
/// LEARN(traits): `Display` and `FromStr` are traits. We implement them so this enum plugs into
/// `parse()`, `format!`, and the HTTP path.
impl FromStr for Category {
    type Err = SpendError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // LEARN(match): `match` must cover every string we care about. The `_` arm is the rest.
        match text.trim().to_ascii_lowercase().as_str() {
            "food" => Ok(Category::Food),
            "transport" => Ok(Category::Transport),
            "rent" => Ok(Category::Rent),
            "utilities" => Ok(Category::Utilities),
            "fun" => Ok(Category::Fun),
            "health" => Ok(Category::Health),
            "other" => Ok(Category::Other),
            _ => Err(SpendError::UnknownCategory(text.trim().to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_case_insensitively() {
        assert_eq!("food".parse::<Category>(), Ok(Category::Food));
        assert_eq!("FOOD".parse::<Category>(), Ok(Category::Food));
        assert_eq!(" Transport ".parse::<Category>(), Ok(Category::Transport));
        assert!("spaceship".parse::<Category>().is_err());
    }

    #[test]
    fn display_matches_serde() {
        for category in Category::ALL {
            let label = category.to_string();
            assert_eq!(label.parse::<Category>(), Ok(category));
            let json = serde_json::to_string(&category).expect("serialize");
            assert_eq!(json, format!("\"{label}\""));
        }
    }
}
