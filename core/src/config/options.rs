pub struct Options {
    pub env: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self { env: false }
    }
}

pub trait Option {
    fn apply<T>(self, opts: &mut Options);
}

pub struct UseEnv(bool);

impl Option for UseEnv {
    fn apply<T>(self, opts: &mut Options) {
        opts.env = true;
    }
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
