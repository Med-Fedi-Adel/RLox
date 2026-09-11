pub type ExprId = usize;

pub struct ExprIdGenerator {
    next: ExprId,
}

impl ExprIdGenerator {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn next(&mut self) -> ExprId {
        let id = self.next;
        self.next += 1;
        id
    }
}
