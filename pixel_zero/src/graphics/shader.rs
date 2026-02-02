pub(crate) struct Shader {
    program: u32,
}

impl Shader {
    pub fn load(vertex: &str, fragment: &str) -> anyhow::Result<Self> {
        Err(anyhow::anyhow!(""))
    }
}
