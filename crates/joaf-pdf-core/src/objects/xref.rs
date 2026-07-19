use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XrefEntry {
    pub byte_offset: usize,
    pub generation: u16,
    pub in_use: bool,
}

pub type XrefTable = BTreeMap<u32, XrefEntry>;
