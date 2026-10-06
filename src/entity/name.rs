//! Minimal multilingual name representation for Phase 1.
//!
//! Language metadata is intentionally opaque. The Knowledge Graph preserves
//! the supplied value and language marker but performs no linguistic analysis.

/// A name/representation associated with an entity.
///
/// `language` is intentionally a minimal opaque marker. It can represent
/// values such as `ar`, `en`, `ur`, or another caller-defined language marker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Name {
    value: String,
    language: String,
}

impl Name {
    /// Constructs a name while preserving its value and language marker.
    #[must_use]
    pub fn new(value: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            language: language.into(),
        }
    }

    /// Returns the represented name text exactly as supplied.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the opaque language marker exactly as supplied.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }
}

#[cfg(test)]
mod tests {
    use super::Name;

    #[test]
    fn name_preserves_value_and_language_metadata() {
        let name = Name::new("الله", "ar");

        assert_eq!(name.value(), "الله");
        assert_eq!(name.language(), "ar");
    }

    #[test]
    fn name_can_represent_multiple_languages_without_interpretation() {
        let arabic = Name::new("محمد", "ar");
        let english = Name::new("Muhammad", "en");
        let urdu = Name::new("محمد", "ur");

        assert_eq!(arabic.value(), "محمد");
        assert_eq!(arabic.language(), "ar");
        assert_eq!(english.value(), "Muhammad");
        assert_eq!(english.language(), "en");
        assert_eq!(urdu.value(), "محمد");
        assert_eq!(urdu.language(), "ur");
    }

    #[test]
    fn name_keeps_language_marker_opaque() {
        let name = Name::new("Example", "custom-language-marker");

        assert_eq!(name.language(), "custom-language-marker");
    }
}
