use crate::graphics::Graphics;

#[unsafe(no_mangle)]
pub extern "C" fn graphics_create() -> *mut Graphics {
    let graphics = Graphics::load();
    graphics.map_or(std::ptr::null_mut(), |gfx| Box::into_raw(Box::new(gfx)))
}

#[unsafe(no_mangle)]
pub extern "C" fn graphics_free(graphics: *mut Graphics) {
    if graphics.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(graphics);
    }
}
