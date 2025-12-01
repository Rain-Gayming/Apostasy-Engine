use derive_more::From;

use crate::utils::slotmap::Key;

#[derive(Clone, Copy, Debug, From, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub Key);
