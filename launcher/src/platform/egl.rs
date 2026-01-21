use gbm::AsRaw as _;
use glam::UVec2;
use khronos_egl::{self as egl, Config, Context, Display, Instance, Static, Surface};

use crate::platform::gbm::Gbm;

pub struct Egl {
    instance: Instance<Static>,
    display: Display,
    config: Config,
    context: Context,
    surface: Surface,
}

impl Egl {
    pub fn load(gbm: &Gbm, size: UVec2) -> anyhow::Result<Self> {
        let instance = Instance::new(Static);
        let display = unsafe { instance.get_display(gbm.device().as_raw() as *mut _) }
            .ok_or(anyhow::anyhow!("No EGL Display found"))?;
        let egl_version = instance.initialize(display)?;
        instance.bind_api(egl::OPENGL_ES_API)?;

        let config_attributes = [
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES2_BIT,
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::NONE,
        ];

        let mut configs = Vec::with_capacity(8);
        instance.choose_config(display, &config_attributes, &mut configs)?;

        let config = *configs
            .iter()
            .find(|&c| {
                let buffer_size = instance
                    .get_config_attrib(display, *c, egl::BUFFER_SIZE)
                    .unwrap_or_default();
                buffer_size == 24
            })
            .or_else(|| configs.first())
            .ok_or(anyhow::anyhow!("No suitable EGL config found",))?;

        let context_attributes = [
            egl::CONTEXT_MAJOR_VERSION,
            3,
            egl::CONTEXT_MINOR_VERSION,
            2,
            egl::NONE,
        ];

        let context = instance.create_context(display, config, None, &context_attributes)?;
        let surface = unsafe {
            instance.create_window_surface(display, config, gbm.surface().as_raw() as *mut _, None)
        }?;
        instance.make_current(display, Some(surface), Some(surface), Some(context))?;

        gl::load_with(|s| instance.get_proc_address(s).unwrap() as *const _);

        unsafe { gl::Viewport(0, 0, size.x as i32, size.y as i32) };

        instance.swap_buffers(display, surface)?;

        Ok(Self {
            instance,
            display,
            config,
            context,
            surface,
        })
    }

    pub fn instance(&self) -> &Instance<Static> {
        &self.instance
    }

    pub fn display(&self) -> Display {
        self.display
    }

    pub fn surface(&self) -> Surface {
        self.surface
    }
}
