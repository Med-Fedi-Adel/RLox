#[derive(Debug)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
