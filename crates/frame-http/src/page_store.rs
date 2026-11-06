//! The page currently being served, shared between the renderer and the HTTP thread.

use std::sync::{Arc, Mutex};

use crate::rendered_page::RenderedPage;

/// Holds the latest rendered page. Readers get a cheap shared handle, so a slow client never
/// blocks a new render.
#[derive(Debug, Default)]
pub struct PageStore {
    current: Mutex<Option<Arc<RenderedPage>>>,
}

impl PageStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the served page.
    pub fn set(&self, page: RenderedPage) {
        // A poisoned lock means a thread panicked mid-update; its contents are not trusted
        // and the page is simply not replaced.
        if let Ok(mut current) = self.current.lock() {
            *current = Some(Arc::new(page));
        }
    }

    /// Returns the served page, or `None` before the first render.
    #[must_use]
    pub fn get(&self) -> Option<Arc<RenderedPage>> {
        self.current.lock().ok().and_then(|current| current.clone())
    }
}
