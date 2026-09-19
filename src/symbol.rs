//! Interned Scheme symbols.

use internment::Intern;

/// An interned symbol name.
///
/// Equal names share one allocation, so comparing and hashing symbols is a
/// pointer operation. Interned names are never freed.
pub(crate) type Symbol = Intern<str>;
