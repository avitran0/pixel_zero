use gbm::AsRaw as _;
use glow::HasContext as _;
use khronos_egl::{self as egl, Config, Context, Display, Instance, Static, Surface};

use crate::graphics::gbm::Gbm;

pub(crate) struct Egl {
    instance: Instance<Static>,
    display: Display,
    _config: Config,
    _context: Context,
    surface: Surface,
    gl: glow::Context,
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
        2,
        egl::CONTEXT_MINOR_VERSION,
        0,
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
        instance.swap_interval(display, 0)?;

        let mut gl = unsafe {
            glow::Context::from_loader_function(|s| {
                instance.get_proc_address(s).unwrap() as *const _
            })
        };
        unsafe {
            gl.viewport(0, 0, gbm.size().x.cast_signed(), gbm.size().y.cast_signed());
        }

        let extensions = unsafe { gl.get_parameter_string(glow::EXTENSIONS) };
        let extensions: Vec<_> = extensions.split_ascii_whitespace().collect();

        let has_debug = extensions.contains(&"KHR_debug") || extensions.contains(&"GL_KHR_debug");
        if has_debug {
            log::info!("debug extension found");
            setup_debug_callback(&mut gl);
        }

        instance.swap_buffers(display, surface)?;

        Ok(Self {
            instance,
            display,
            _config: config,
            _context: context,
            surface,
            gl,
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

    pub(crate) fn gl(&self) -> &glow::Context {
        &self.gl
    }
}

fn setup_debug_callback(gl: &mut glow::Context) {
    fn debug_callback(source: u32, kind: u32, _id: u32, severity: u32, message: &str) {
        let source = match source {
            glow::DEBUG_SOURCE_API => "API",
            glow::DEBUG_SOURCE_APPLICATION => "Application",
            glow::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
            glow::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
            glow::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
            glow::DEBUG_SOURCE_OTHER => "Other",
            _ => "Unknown",
        };

        let kind = match kind {
            glow::DEBUG_TYPE_ERROR => "Error",
            glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
            glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
            glow::DEBUG_TYPE_PORTABILITY => "Portability",
            glow::DEBUG_TYPE_PERFORMANCE => "Performance",
            glow::DEBUG_TYPE_MARKER => "Marker",
            glow::DEBUG_TYPE_OTHER => "Other",
            _ => "Unknown",
        };

        match severity {
            glow::DEBUG_SEVERITY_HIGH => log::error!("[{source}/{kind}] {message}"),
            glow::DEBUG_SEVERITY_MEDIUM => log::warn!("[{source}/{kind}] {message}"),
            _ => {}
        }
    }

    unsafe {
        gl.enable(glow::DEBUG_OUTPUT);
        gl.debug_message_callback(debug_callback);

        gl.debug_message_control(
            glow::DONT_CARE,
            glow::DONT_CARE,
            glow::DEBUG_SEVERITY_NOTIFICATION,
            &[],
            false,
        );
    }
}
