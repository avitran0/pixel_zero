use gbm::AsRaw as _;
use khronos_egl::{self as egl, Config, Context, Display, Instance, Static, Surface};

use crate::graphics::gbm::Gbm;

pub(crate) struct Egl {
    instance: Instance<Static>,
    display: Display,
    _config: Config,
    _context: Context,
    surface: Surface,
}

impl Egl {
    const CONFIG_ATTRIBUTES: [i32; 13] = [
        egl::RED_SIZE,
        8,
        egl::GREEN_SIZE,
        8,
        egl::BLUE_SIZE,
        8,
        egl::ALPHA_SIZE,
        8,
        egl::RENDERABLE_TYPE,
        egl::OPENGL_ES3_BIT,
        egl::SURFACE_TYPE,
        egl::WINDOW_BIT,
        egl::NONE,
    ];

    const CONTEXT_ATTRIBUTES: [i32; 5] = [
        egl::CONTEXT_MAJOR_VERSION,
        3,
        egl::CONTEXT_MINOR_VERSION,
        2,
        egl::NONE,
    ];

    pub(crate) fn load(gbm: &mut Gbm) -> Result<Self, egl::Error> {
        let instance = Instance::new(Static);
        let display = unsafe { instance.get_display(gbm.device().as_raw() as *mut _) }
            .ok_or(egl::Error::BadDisplay)?;
        let (major, minor) = instance.initialize(display)?;
        log::info!("egl version {major}.{minor}");
        instance.bind_api(egl::OPENGL_ES_API)?;

        let mut configs = Vec::with_capacity(8);
        instance.choose_config(display, &Self::CONFIG_ATTRIBUTES, &mut configs)?;

        let config = *configs.first().ok_or(egl::Error::BadConfig)?;

        let visual_id = instance.get_config_attrib(display, config, egl::NATIVE_VISUAL_ID)?;
        let gbm_format = unsafe { std::mem::transmute::<i32, gbm::Format>(visual_id) };

        gbm.init_surface(gbm_format)
            .map_err(|_| egl::Error::BadSurface)?;

        let context = instance.create_context(display, config, None, &Self::CONTEXT_ATTRIBUTES)?;
        let surface = unsafe {
            instance.create_window_surface(display, config, gbm.surface().as_raw() as *mut _, None)
        }?;
        instance.make_current(display, Some(surface), Some(surface), Some(context))?;

        gl::load_with(|s| instance.get_proc_address(s).unwrap() as *const _);

        unsafe { gl::Viewport(0, 0, gbm.size().x.cast_signed(), gbm.size().y.cast_signed()) };

        instance.swap_buffers(display, surface)?;

        Ok(Self {
            instance,
            display,
            _config: config,
            _context: context,
            surface,
        })
    }

    pub(crate) fn instance(&self) -> &Instance<Static> {
        &self.instance
    }

    pub(crate) fn display(&self) -> Display {
        self.display
    }

    pub(crate) fn surface(&self) -> Surface {
        self.surface
    }
}
