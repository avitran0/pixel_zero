use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use parking_lot::Mutex;

use crate::graphics::Graphics;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    pub fn new(source: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug, Clone)]
pub struct Ui(Arc<Mutex<UiInner>>);

impl Ui {
    pub fn start_frame(&self) -> UiFrame {
        UiFrame {
            context: self.clone(),
        }
    }
}

#[derive(Debug)]
pub struct UiInner {}

impl UiInner {}

pub struct UiFrame {
    context: Ui,
}

impl UiFrame {
    pub fn render(self, graphics: &Graphics) {}
}
