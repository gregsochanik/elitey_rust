/// What a star's spectral class says about it
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarClass {
    pub code: String,
    pub type_name: String,
    pub note: &'static str,
}

impl StarClass {
    /// Read a class such as `G2V`, `M3.5` or `DA`; None when the letter is unknown
    pub fn from_code(code: &str) -> Option<Self> {
        let code = code.trim();
        let letter = code.chars().next()?.to_ascii_uppercase();
        let size = Self::size(code);

        let (type_name, note) = match (letter, size) {
            ('D', _) => (
                "White dwarf".to_string(),
                "The dead core of a sun, as small as a planet but a million times denser.",
            ),
            ('L' | 'T', _) => (
                "Brown dwarf".to_string(),
                "Too small to ignite fully; a failed star glowing faintly from its own heat.",
            ),
            (_, Size::Giant) => (
                format!("{} giant", Self::colour(letter)?),
                "Swollen near the end of its life, hundreds of times wider than Sol.",
            ),
            (_, Size::Subgiant) => (
                format!("{} subgiant", Self::colour(letter)?),
                "Running low on hydrogen and beginning to swell.",
            ),
            (_, Size::Supergiant) => (
                format!("{} supergiant", Self::colour(letter)?),
                "A rare, enormous star that will end as a supernova.",
            ),
            ('O', _) => (
                "Blue main-sequence star".to_string(),
                "Over 30,000 K and burning fast; rare, and gone in a few million years.",
            ),
            ('B', _) => (
                "Blue-white main-sequence star".to_string(),
                "10,000 to 30,000 K, a hot young star that lives for only a few hundred million years.",
            ),
            ('A', _) => (
                "White main-sequence star".to_string(),
                "7,500 to 10,000 K, like Sirius: bright, hot and short-lived.",
            ),
            ('F', _) => (
                "Yellow-white main-sequence star".to_string(),
                "6,000 to 7,500 K, a little hotter and brighter than Sol.",
            ),
            ('G', _) => (
                "Yellow dwarf".to_string(),
                "5,200 to 6,000 K, a star like Sol that burns steadily for about ten billion years.",
            ),
            ('K', _) => (
                "Orange dwarf".to_string(),
                "3,700 to 5,200 K, dimmer than Sol but long-lived and calm, a kind friend to planets.",
            ),
            ('M', _) => (
                "Red dwarf".to_string(),
                "Under 3,700 K, small and dim; the most common star in the galaxy, and prone to flares.",
            ),
            _ => return None,
        };
        Some(Self {
            code: code.to_string(),
            type_name,
            note,
        })
    }

    fn colour(letter: char) -> Option<&'static str> {
        match letter {
            'O' | 'B' => Some("Blue"),
            'A' => Some("White"),
            'F' => Some("Yellow-white"),
            'G' => Some("Yellow"),
            'K' => Some("Orange"),
            'M' => Some("Red"),
            _ => None,
        }
    }

    fn size(code: &str) -> Size {
        let numeral: String = code
            .chars()
            .skip(1)
            .skip_while(|c| c.is_ascii_digit() || *c == '.')
            .take_while(|c| matches!(c, 'I' | 'V'))
            .collect();
        match numeral.as_str() {
            "I" | "II" => Size::Supergiant,
            "III" => Size::Giant,
            "IV" => Size::Subgiant,
            _ => Size::Dwarf,
        }
    }
}

enum Size {
    Dwarf,
    Subgiant,
    Giant,
    Supergiant,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_of(code: &str) -> String {
        StarClass::from_code(code).unwrap().type_name
    }

    #[test]
    fn main_sequence_classes_get_a_colour_and_size() {
        assert_eq!(type_of("G2V"), "Yellow dwarf");
        assert_eq!(type_of("K5"), "Orange dwarf");
        assert_eq!(type_of("M3.5"), "Red dwarf");
        assert_eq!(type_of("A1V"), "White main-sequence star");
    }

    #[test]
    fn luminosity_classes_make_giants() {
        assert_eq!(type_of("K2III"), "Orange giant");
        assert_eq!(type_of("M2I"), "Red supergiant");
        assert_eq!(type_of("F5IV"), "Yellow-white subgiant");
    }

    #[test]
    fn dead_and_failed_stars_are_named() {
        assert_eq!(type_of("DA"), "White dwarf");
        assert_eq!(type_of("L5"), "Brown dwarf");
    }

    #[test]
    fn an_unknown_class_is_none() {
        assert!(StarClass::from_code("").is_none());
        assert!(StarClass::from_code("X9").is_none());
    }
}
