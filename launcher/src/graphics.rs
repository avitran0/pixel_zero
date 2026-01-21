use std::{
    fs::File,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::fs::FileTypeExt,
    },
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use api::graphics::Graphics;
use drm::{
    Device as DrmDevice,
    control::{
        Device as ControlDevice, Event, Mode, ModeTypeFlags, PageFlipFlags, connector, crtc,
        framebuffer,
    },
};
use gbm::{
    AsRaw as _, BufferObject, BufferObjectFlags, Device as GbmDevice, Surface as GbmSurface,
};
use glam::UVec2;
use khronos_egl as egl;

pub struct GraphicsContext {
    drm: Drm,
    gbm: Gbm,
    egl: Egl,

    framebuffer: Option<framebuffer::Handle>,
    buffer_object: Option<BufferObject<()>>,
}

static LOADED: AtomicBool = AtomicBool::new(false);
impl GraphicsContext {
    pub fn load() -> anyhow::Result<Self> {
        if LOADED.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("GraphicsContext already loaded"));
        }

        let drm = Drm::load()?;
        let gbm = Gbm::load(&drm)?;
        let egl = Egl::load(&gbm)?;

        let buffer_object = unsafe { gbm.surface.lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let framebuffer = drm.gpu.add_framebuffer(&buffer_object, bpp, bpp)?;
        drm.gpu.set_crtc(
            drm.crtc.handle(),
            Some(framebuffer),
            (0, 0),
            &[drm.connector.handle()],
            Some(drm.mode),
        )?;

        LOADED.store(true, Ordering::Relaxed);

        Ok(Self {
            drm,
            gbm,
            egl,
            framebuffer: Some(framebuffer),
            buffer_object: Some(buffer_object),
        })
    }

    pub fn clear(&self) {
        unsafe {
            gl::ClearColor(0.2, 0.5, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT)
        };
    }

    pub fn present(&mut self) -> anyhow::Result<()> {
        self.egl
            .egl
            .swap_buffers(self.egl.display, self.egl.surface)?;

        let buffer_object = unsafe { self.gbm.surface.lock_front_buffer() }?;
        let bpp = buffer_object.bpp();
        let framebuffer = self.drm.gpu.add_framebuffer(&buffer_object, bpp, bpp)?;

        self.drm.gpu.page_flip(
            self.drm.crtc.handle(),
            framebuffer,
            PageFlipFlags::EVENT,
            None,
        )?;
        let events = self.drm.gpu.receive_events()?;
        for event in events {
            if let Event::PageFlip(event) = event {
                // todo
            }
        }

        if let Some(framebuffer) = &self.framebuffer {
            self.drm.gpu.destroy_framebuffer(*framebuffer)?;
        }

        self.buffer_object = Some(buffer_object);
        self.framebuffer = Some(framebuffer);

        Ok(())
    }
}

impl Graphics for GraphicsContext {
    fn clear(&self, color: api::graphics::Color) {}
}

struct Drm {
    gpu: Arc<Gpu>,
    connector: connector::Info,
    mode: Mode,
    crtc: crtc::Info,
}

impl Drm {
    fn load() -> anyhow::Result<Self> {
        let gpu = Gpu::open()?;

        let resources = gpu.resource_handles()?;

        let Some(connector) = resources
            .connectors()
            .iter()
            .flat_map(|handle| gpu.get_connector(*handle, true))
            .find(|connector| connector.state() == connector::State::Connected)
        else {
            return Err(anyhow::anyhow!("No connected connectors found"));
        };

        let mode = *connector
            .modes()
            .iter()
            .find(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
            .unwrap_or_else(|| &connector.modes()[0]);

        let Some(crtc) = connector
            .encoders()
            .iter()
            .flat_map(|handle| gpu.get_encoder(*handle))
            .flat_map(|encoder| encoder.crtc())
            .flat_map(|crtc| gpu.get_crtc(crtc))
            .next()
        else {
            return Err(anyhow::anyhow!("No suitable CRTC found"));
        };

        Ok(Self {
            gpu: Arc::new(gpu),
            connector,
            mode,
            crtc,
        })
    }

    fn size(&self) -> UVec2 {
        UVec2 {
            x: self.mode.size().0 as u32,
            y: self.mode.size().1 as u32,
        }
    }

    fn gpu(&self) -> Arc<Gpu> {
        self.gpu.clone()
    }
}

struct Gbm {
    device: GbmDevice<Arc<Gpu>>,
    surface: GbmSurface<()>,
}

impl Gbm {
    pub fn load(drm: &Drm) -> anyhow::Result<Self> {
        let size = drm.size();
        let device = GbmDevice::new(drm.gpu())?;
        let surface = device.create_surface(
            size.x,
            size.y,
            gbm::Format::Xrgb8888,
            BufferObjectFlags::SCANOUT | BufferObjectFlags::RENDERING,
        )?;
        Ok(Self { device, surface })
    }
}

struct Egl {
    egl: egl::Instance<egl::Static>,
    display: egl::Display,
    config: egl::Config,
    context: egl::Context,
    surface: egl::Surface,
}

impl Egl {
    pub fn load(gbm: &Gbm) -> anyhow::Result<Self> {
        let egl = egl::Instance::new(egl::Static);
        let display = unsafe { egl.get_display(gbm.device.as_raw() as *mut _) }
            .ok_or(anyhow::anyhow!("No EGL Display found"))?;
        let egl_version = egl.initialize(display)?;
        egl.bind_api(egl::OPENGL_ES_API)?;

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
        egl.choose_config(display, &config_attributes, &mut configs)?;

        let config = configs
            .into_iter()
            .find(|c| {
                let buffer_size = egl
                    .get_config_attrib(display, *c, egl::BUFFER_SIZE)
                    .unwrap_or_default();
                buffer_size == 24
            })
            .ok_or(anyhow::anyhow!("No suitable EGL config found",))?;

        let context_attributes = [
            egl::CONTEXT_MAJOR_VERSION,
            3,
            egl::CONTEXT_MINOR_VERSION,
            2,
            egl::NONE,
        ];

        let context = egl.create_context(display, config, None, &context_attributes)?;
        let surface = unsafe {
            egl.create_window_surface(display, config, gbm.surface.as_raw() as *mut _, None)
        }?;
        egl.make_current(display, Some(surface), Some(surface), Some(context))?;

        gl::load_with(|s| egl.get_proc_address(s).unwrap() as *const _);

        egl.swap_buffers(display, surface)?;

        Ok(Self {
            egl,
            display,
            config,
            context,
            surface,
        })
    }
}

struct Gpu {
    file: File,
}

impl Gpu {
    pub fn open() -> std::io::Result<Self> {
        let dir = std::fs::read_dir("/dev/dri")?;
        for file in dir {
            let file = file?;
            if !file.file_type()?.is_char_device() {
                continue;
            }
            let name_osstr = file.file_name();
            let Some(name) = name_osstr.to_str() else {
                continue;
            };
            if !name.starts_with("card") {
                continue;
            }
            let file = File::options().write(true).read(true).open(file.path())?;
            return Ok(Self { file });
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No valid DRM device found",
        ))
    }
}

impl AsFd for Gpu {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

impl DrmDevice for Gpu {}
impl ControlDevice for Gpu {}

pub fn resolutions() -> std::io::Result<Vec<String>> {
    let card = Gpu::open()?;
    let resources = card.resource_handles()?;
    let Some(connector) = resources
        .connectors()
        .iter()
        .flat_map(|handle| card.get_connector(*handle, true))
        .find(|connector| connector.state() == connector::State::Connected)
    else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No connected connectors found",
        ));
    };
    Ok(connector
        .modes()
        .iter()
        .map(|mode| format!("{}x{} {}hz", mode.size().0, mode.size().1, mode.vrefresh()))
        .collect())
}
