use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(usize);

impl Id {
    pub fn raw(&self) -> usize {
        self.0
    }
}

pub struct IdGenerator {
    next_id: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    pub fn gen(&mut self) -> Id {
        let id = Id(self.next_id);
        self.next_id += 1;
        id
    }
}
