use egui::{Context, RawInput};

pub struct Egui {
    ctx: Context,
}

impl Egui {
    pub fn load() -> anyhow::Result<Self> {
        let ctx = Context::default();
    }

    pub fn run_ui(&self, run_ui: impl FnMut(&Context)) {
        let output = self.ctx.run(self.input.clone(), run_ui);
        let clipped_primitives = self
            .ctx
            .tessellate(output.shapes, output.pixels_per_point);
    }
}
