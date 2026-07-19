use crate::Page;

pub struct Catalog<'a> {
    pub pages: Vec<Page<'a>>,
}

impl<'a> Catalog<'a> {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }
}
