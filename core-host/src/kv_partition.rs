#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicSwap {
    pub deletes: Vec<String>,
    pub inserts: Vec<(String, Vec<u8>)>,
}

impl AtomicSwap {
    pub fn is_noop(&self) -> bool {
        self.deletes.is_empty() && self.inserts.is_empty()
    }
}
