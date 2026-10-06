//! Alias representation for the Phase 1 entity model.
//!
//! Aliases are alternative names of an entity, not independent semantic
//! objects. Therefore Phase 1 deliberately does not define an `AliasId` or a
//! separate alias identity system.

use super::name::Name;

/// An alias is represented by the same minimal representation as a [`Name`].
///
/// Keeping `Alias` as a type alias makes the ownership boundary explicit:
/// aliases remain part of an entity's name/representation model rather than
/// becoming a separate semantic object.
pub type Alias = Name;

#[cfg(test)]
mod tests {
    use super::Alias;

    #[test]
    fn alias_uses_the_name_representation_boundary() {
        let alias = Alias::new("Allah", "en");

        assert_eq!(alias.value(), "Allah");
        assert_eq!(alias.language(), "en");
    }

    #[test]
    fn multiple_aliases_can_be_represented() {
        let aliases = [
            Alias::new("Allah", "en"),
            Alias::new("اللہ", "ur"),
            Alias::new("الله", "ar"),
        ];

        assert_eq!(aliases.len(), 3);
        assert_eq!(aliases[0].value(), "Allah");
        assert_eq!(aliases[1].value(), "اللہ");
        assert_eq!(aliases[2].value(), "الله");
    }
}
