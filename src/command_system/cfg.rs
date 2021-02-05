#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum CaseSensitivity {
    False = 0,
    True = 1,
}

pub use CaseSensitivity::{True as CaseSensitive, False as CaseInsensitive};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum UseFullyQualifiedName {
    False = 0,
    True = 1,
}

pub use UseFullyQualifiedName::{True as FullyQualifiedName, False as NoFullyQualifiedName};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum EnabledAliases {
    False = 0,
    True = 1,
}

pub use EnabledAliases::{True as EnableAliases, False as DisableAliases};