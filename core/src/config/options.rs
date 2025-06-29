pub struct Options {
    pub env: bool,
}

impl Options {
    pub fn new() -> Self {
        Options { env: false }
    }

    pub fn use_env(mut self) -> Self {
        self.env = true;
        self
    }

    pub fn build(self) -> Options {
        self
    }
}

pub trait Option {
    fn apply(&self, opts: &mut Options);
}


fn main() {
    // 使用示例

    let opts = Options::new().use_env().build();
}
