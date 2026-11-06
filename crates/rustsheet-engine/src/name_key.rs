//! Keys of defined names, matched without regard to case and looked up without allocating
//! (T-1194).
//!
//! A name is found under its scope and its name lower-cased as [`str::to_lowercase`] does it.
//! A lookup folds the name as written one `char` at a time while it is hashed and compared, so
//! no lower-cased copy is built: a name used in reference context is looked up several times
//! (`Model::is_multi_area`, `Model::defined_name_reference`, the value path), and each lookup
//! used to allocate one.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A stored key: the scope (the sheet index, `None` for a workbook name) and the name
/// lower-cased.
#[derive(Clone, Debug)]
pub(crate) struct NameKey {
    scope: Option<u32>,
    lower: String,
}

impl NameKey {
    /// The key of `name` in `scope`.
    pub(crate) fn new(scope: Option<u32>, name: &str) -> Self {
        Self {
            scope,
            lower: name.to_lowercase(),
        }
    }

    /// The scope: the sheet index, `None` for a workbook name.
    pub(crate) fn scope(&self) -> Option<u32> {
        self.scope
    }

    /// The name, lower-cased.
    pub(crate) fn lower(&self) -> &str {
        &self.lower
    }
}

/// A key as hashed and compared: its scope, its name, and whether that name is lower-cased
/// already (a stored key) or still to be folded (a lookup).
trait KeyParts {
    fn parts(&self) -> (Option<u32>, &str, bool);
}

impl KeyParts for NameKey {
    fn parts(&self) -> (Option<u32>, &str, bool) {
        (self.scope, &self.lower, true)
    }
}

/// A name as written, looked up in `scope`.
struct Written<'a> {
    scope: Option<u32>,
    name: &'a str,
}

impl KeyParts for Written<'_> {
    fn parts(&self) -> (Option<u32>, &str, bool) {
        (self.scope, self.name, false)
    }
}

/// The lower-cased characters of `name`: as they are when `lower`, else folded one by one.
fn folded(name: &str, lower: bool) -> impl Iterator<Item = char> + '_ {
    let (stored, written) = if lower {
        (Some(name.chars()), None)
    } else {
        (None, Some(name.chars().flat_map(char::to_lowercase)))
    };
    stored
        .into_iter()
        .flatten()
        .chain(written.into_iter().flatten())
}

impl Hash for dyn KeyParts + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (scope, name, lower) = self.parts();
        scope.hash(state);
        for c in folded(name, lower) {
            state.write_u32(u32::from(c));
        }
        // Not a `char`, so no name's hash input is a prefix of another's.
        state.write_u32(u32::MAX);
    }
}

impl PartialEq for dyn KeyParts + '_ {
    fn eq(&self, other: &Self) -> bool {
        let (scope, name, lower) = self.parts();
        let (other_scope, other_name, other_lower) = other.parts();
        scope == other_scope && folded(name, lower).eq(folded(other_name, other_lower))
    }
}

impl Eq for dyn KeyParts + '_ {}

// A stored key hashes and compares as its `dyn KeyParts`, so a lookup may borrow it as one.
impl Hash for NameKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as &dyn KeyParts).hash(state);
    }
}

impl PartialEq for NameKey {
    fn eq(&self, other: &Self) -> bool {
        self.scope == other.scope && self.lower == other.lower
    }
}

impl Eq for NameKey {}

impl<'a> Borrow<dyn KeyParts + 'a> for NameKey {
    fn borrow(&self) -> &(dyn KeyParts + 'a) {
        self
    }
}

/// The entry of `map` for the name `name` (as written, any case) in `scope`.
pub(crate) fn get_name<'m, V>(
    map: &'m HashMap<NameKey, V>,
    scope: Option<u32>,
    name: &str,
) -> Option<&'m V> {
    // `str::to_lowercase` lowers a capital sigma by its place in the word (`ς` at the end, `σ`
    // elsewhere), which one `char` at a time cannot: such a name is lower-cased whole.
    if name.contains('Σ') {
        return map.get(&NameKey::new(scope, name));
    }
    map.get(&Written { scope, name } as &dyn KeyParts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_found_in_any_case_in_its_own_scope() {
        let mut map = HashMap::new();
        map.insert(NameKey::new(None, "Rates"), 1);
        map.insert(NameKey::new(Some(0), "rates"), 2);
        map.insert(NameKey::new(None, "ÄRGER"), 3);
        map.insert(NameKey::new(None, "ΟΔΟΣ"), 4);
        map.insert(NameKey::new(None, "İx"), 5);
        assert_eq!(get_name(&map, None, "RATES"), Some(&1));
        assert_eq!(get_name(&map, None, "rAtEs"), Some(&1));
        assert_eq!(get_name(&map, Some(0), "Rates"), Some(&2));
        assert_eq!(get_name(&map, Some(1), "Rates"), None);
        assert_eq!(get_name(&map, None, "Rate"), None);
        assert_eq!(get_name(&map, None, "ratess"), None);
        assert_eq!(get_name(&map, None, "ärger"), Some(&3));
        // A capital sigma lowers to `ς` at the end of the word, as `str::to_lowercase` has it.
        assert_eq!(get_name(&map, None, "ΟΔΟΣ"), Some(&4));
        assert_eq!(get_name(&map, None, "οδος"), Some(&4));
        assert_eq!(get_name(&map, None, "οδοσ"), None);
        assert_eq!(get_name(&map, None, "İX"), Some(&5));
    }
}
