use std::{collections::BTreeMap, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XrefEntry {
    pub byte_offset: usize,
    pub generation: u16,
    pub in_use: bool,
}

pub type XrefTable = BTreeMap<u32, XrefEntry>;

impl Display for XrefEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:020} {:05} {}",
            self.byte_offset,
            self.generation,
            if self.in_use { "n" } else { "f" }
        )
    }
}
