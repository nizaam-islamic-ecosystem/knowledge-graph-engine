//! Deterministic normalization and matching primitives for Phase 3 entity
//! resolution.
//!
//! The Knowledge Graph owns only generic structural normalization here. It
//! does not perform Arabic morphology, stemming, root extraction, transliteration
//! generation, or any other language-specific linguistic analysis. A caller
//! such as the Arabic Engine may provide linguistically prepared forms which
//! this module can compare deterministically.

use core::fmt;

/// Structural normalization failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NormalizationError {
    /// The input contains a Unicode control character.
    ControlCharacter { index: usize },
}

impl fmt::Display for NormalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ControlCharacter { index } => write!(
                formatter,
                "normalization input contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for NormalizationError {}

/// Deterministically normalizes generic textual input.
///
/// This performs only language-neutral structural work:
/// - surrounding whitespace is removed;
/// - Unicode whitespace runs collapse to one ASCII space;
/// - Unicode alphabetic/digit characters are retained;
/// - punctuation/symbol separators become spaces;
/// - Unicode scalar values are lower-cased using Rust's language-neutral
///   Unicode case mapping.
///
/// Language-specific normalization remains outside the Knowledge Graph.
pub fn normalize(input: &str) -> Result<String, NormalizationError> {
    if let Some(index) = input
        .chars()
        .position(|character| character.is_control() && !character.is_whitespace())
    {
        return Err(NormalizationError::ControlCharacter { index });
    }

    let mut output = String::new();
    let mut pending_space = false;

    for character in input.chars() {
        if character.is_whitespace() {
            pending_space = !output.is_empty();
            continue;
        }

        if character.is_alphanumeric() || is_mark(character) {
            if pending_space && !output.is_empty() {
                output.push(' ');
            }
            pending_space = false;
            output.extend(character.to_lowercase());
        } else {
            pending_space = !output.is_empty();
        }
    }

    Ok(output.trim().to_owned())
}

/// Returns whether a character belongs to one of the Unicode combining-mark
/// ranges handled by the generic normalizer. Rust's stable `char` API exposes
/// alphabetic and numeric classification directly, but not the Unicode
/// General Category `M` as a single predicate. These ranges cover the common
/// combining-mark blocks, including Arabic, while keeping the normalizer
/// language-neutral and dependency-free.
#[must_use]
fn is_mark(character: char) -> bool {
    let code_point = character as u32;

    matches!(
        code_point,
        0x0300..=0x036f
            | 0x0483..=0x0489
            | 0x0591..=0x05bd
            | 0x05bf
            | 0x05c1..=0x05c2
            | 0x05c4..=0x05c5
            | 0x05c7
            | 0x0610..=0x061a
            | 0x064b..=0x065f
            | 0x0670
            | 0x06d6..=0x06dc
            | 0x06df..=0x06e4
            | 0x06e7..=0x06e8
            | 0x06ea..=0x06ed
            | 0x0711
            | 0x0730..=0x074a
            | 0x07a6..=0x07b0
            | 0x07eb..=0x07f3
            | 0x0816..=0x0819
            | 0x081b..=0x0823
            | 0x0825..=0x0827
            | 0x0829..=0x082d
            | 0x0859..=0x085f
            | 0x08d3..=0x08ff
            | 0x1ab0..=0x1aff
            | 0x1dc0..=0x1dff
            | 0x20d0..=0x20ff
            | 0x2de0..=0x2dff
            | 0xfe20..=0xfe2f
            | 0xe0100..=0xe01ef
    )
}

/// Returns whether two values are exact textual matches.
#[must_use]
pub fn exact_match(left: &str, right: &str) -> bool {
    left == right
}

/// Returns whether two values match after generic normalization.
pub fn normalized_match(left: &str, right: &str) -> Result<bool, NormalizationError> {
    Ok(normalize(left)? == normalize(right)?)
}

/// Returns whether a supplied transliteration form matches the supplied
/// transliteration value after generic normalization.
///
/// The function compares transliterations but does not generate them. This is
/// the explicit boundary between KG matching and language-specific linguistic
/// processing.
pub fn transliteration_match(
    reference_transliteration: &str,
    candidate_transliteration: &str,
) -> Result<bool, NormalizationError> {
    normalized_match(reference_transliteration, candidate_transliteration)
}

#[cfg(test)]
mod tests {
    use super::{
        NormalizationError, exact_match, normalize, normalized_match, transliteration_match,
    };

    #[test]
    fn normalization_collapses_whitespace_and_structural_punctuation() {
        let normalized = normalize("  Al-Lah,   Example!  ").expect("valid text");

        assert_eq!(normalized, "al lah example");
    }

    #[test]
    fn normalization_is_case_insensitive_for_latin_text() {
        assert_eq!(normalize("Muhammad").expect("valid text"), "muhammad");
        assert_eq!(normalize("MUHAMMAD").expect("valid text"), "muhammad");
    }

    #[test]
    fn normalization_preserves_non_latin_letters_and_marks() {
        assert_eq!(normalize(" مُحَمَّد ").expect("valid text"), "مُحَمَّد");
    }

    #[test]
    fn exact_matching_does_not_normalize_values() {
        assert!(exact_match("Muhammad", "Muhammad"));
        assert!(!exact_match("Muhammad", " muhammad "));
    }

    #[test]
    fn normalized_matching_accepts_structural_variation() {
        assert!(normalized_match(" Muhammad ", "MUHAMMAD").expect("valid text"));
        assert!(normalized_match("al-bukhari", "al bukhari").expect("valid text"));
    }

    #[test]
    fn transliteration_matching_compares_supplied_forms_only() {
        assert!(transliteration_match("Muhammad", " muhammad ").expect("valid text"));
        assert!(!transliteration_match("Muhammad", "Mohammed").expect("valid text"));
    }

    #[test]
    fn whitespace_controls_are_collapsed_but_non_whitespace_controls_are_rejected() {
        assert_eq!(
            normalize("Muh\nammad\tAli").expect("valid whitespace"),
            "muh ammad ali"
        );
        assert_eq!(
            normalize("Muh\0ammad"),
            Err(NormalizationError::ControlCharacter { index: 3 })
        );
    }

    #[test]
    fn unicode_punctuation_is_treated_as_a_separator_while_marks_are_preserved() {
        assert_eq!(
            normalize("al‑bukhari، مُحَمَّد").expect("valid text"),
            "al bukhari مُحَمَّد"
        );
    }
}
