use crate::Page;

pub struct Catalog {
    pub pages: Vec<Page>,
}

impl Catalog {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }
}
