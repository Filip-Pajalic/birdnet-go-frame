//! The pages being served, shared between the renderer and the HTTP threads.

use std::sync::{Arc, Mutex};

use crate::rendered_page::RenderedPage;
use crate::target::Target;

/// Draws the current page for a target, or returns `None` when it cannot.
pub type PageMaker = dyn Fn(Target) -> Option<RenderedPage> + Send + Sync;

/// Draws the current page for a target and a view, or returns `None` when it cannot. A view is
/// the app's own name for one way of drawing the page, e.g. a frame and its page mode; the
/// store only tells views apart.
pub type ViewMaker = dyn Fn(Target, &str) -> Option<RenderedPage> + Send + Sync;

/// Most pages kept at once: a few frames in a few views, and screens, not every size a client
/// could ask for.
const MOST_PAGES: usize = 24;

/// Holds the current page for every target asked for.
///
/// The renderer publishes a [`PageMaker`] whenever what the page shows changes. A target's
/// page is drawn the first time a client asks for it, then kept until the next publish, so
/// panels that wake every few minutes get the same bytes (and a 304) until the birds change.
/// Readers get a cheap shared handle, so a slow client never blocks a new render.
pub struct PageStore {
    default: Target,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    /// Counts publishes, so a page drawn from an old maker is not cached after a new one.
    generation: u64,
    maker: Option<Arc<ViewMaker>>,
    pages: Vec<((Target, String), Arc<RenderedPage>)>,
}

impl PageStore {
    /// Creates an empty store serving `default` to clients that do not say what they are.
    #[must_use]
    pub fn new(default: Target) -> Self {
        Self {
            default,
            inner: Mutex::new(Inner::default()),
        }
    }

    /// Returns the target served to clients that do not say what they are.
    #[must_use]
    pub fn default_target(&self) -> Target {
        self.default
    }

    /// Replaces what is served with pages drawn by `maker`, the same page in every view,
    /// forgetting every page drawn so far.
    pub fn publish(&self, maker: Box<PageMaker>) {
        self.publish_views(Box::new(move |target, _view| maker(target)));
    }

    /// Replaces what is served with pages drawn by `maker` for each target and view,
    /// forgetting every page drawn so far.
    pub fn publish_views(&self, maker: Box<ViewMaker>) {
        // A poisoned lock means a thread panicked mid-update; its contents are not trusted
        // and the pages are simply not replaced.
        if let Ok(mut inner) = self.inner.lock() {
            inner.generation += 1;
            inner.maker = Some(Arc::from(maker));
            inner.pages.clear();
        }
    }

    /// Serves `page` for the default target only, e.g. in tests.
    pub fn set(&self, page: RenderedPage) {
        let default: Target = self.default;
        let page: Arc<RenderedPage> = Arc::new(page);
        self.publish(Box::new(move |target| {
            (target == default).then(|| RenderedPage::clone(&page))
        }));
    }

    /// Returns whether anything has been published.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.inner.lock().is_ok_and(|inner| inner.maker.is_some())
    }

    /// Returns the default target's page, or `None` before the first publish.
    #[must_use]
    pub fn get(&self) -> Option<Arc<RenderedPage>> {
        self.page(self.default)
    }

    /// Returns `target`'s page in the default view, drawing it if this is the first request
    /// since the last publish. `None` before the first publish, or when the page cannot be
    /// drawn.
    #[must_use]
    pub fn page(&self, target: Target) -> Option<Arc<RenderedPage>> {
        self.view(target, "")
    }

    /// Returns `target`'s page in `view`, drawing it if this is the first request since the
    /// last publish. `None` before the first publish, or when the page cannot be drawn.
    #[must_use]
    pub fn view(&self, target: Target, view: &str) -> Option<Arc<RenderedPage>> {
        let found = |kept: &(Target, String)| kept.0 == target && kept.1 == view;
        let (generation, maker): (u64, Arc<ViewMaker>) = {
            let inner = self.inner.lock().ok()?;
            if let Some((_, page)) = inner.pages.iter().find(|(kept, _)| found(kept)) {
                return Some(Arc::clone(page));
            }
            (inner.generation, Arc::clone(inner.maker.as_ref()?))
        };
        // Drawn without the lock, so other targets are served meanwhile.
        let page: Arc<RenderedPage> = Arc::new(maker(target, view)?);
        if let Ok(mut inner) = self.inner.lock() {
            let current: bool = inner.generation == generation;
            if current && !inner.pages.iter().any(|(kept, _)| found(kept)) {
                if inner.pages.len() >= MOST_PAGES {
                    inner.pages.remove(0);
                }
                inner
                    .pages
                    .push(((target, String::from(view)), Arc::clone(&page)));
            }
        }
        Some(page)
    }
}

impl std::fmt::Debug for PageStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PageStore")
            .field("default", &self.default)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use frame_core::{IndexedImage, Ink, PanelSize, Resolution, Rgb, Rotation};
    use frame_raster::Canvas;

    use super::*;

    fn draw(target: Target) -> Option<RenderedPage> {
        let resolution: Resolution = target.resolution();
        let page = IndexedImage::new(resolution, Ink::White);
        let preview = Canvas::new(resolution, Rgb::WHITE);
        RenderedPage::for_target(target, &page, &preview, 15).ok()
    }

    #[test]
    fn draws_each_target_once_per_publish() {
        let default = Target::Panel {
            panel: PanelSize::Inches13Point3,
            rotation: Rotation::Degrees0,
        };
        let store = PageStore::new(default);
        assert!(store.get().is_none() && !store.is_ready());
        let draws: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        let counter: Arc<AtomicUsize> = Arc::clone(&draws);
        store.publish(Box::new(move |target| {
            counter.fetch_add(1, Ordering::SeqCst);
            draw(target)
        }));
        let small = Target::Panel {
            panel: PanelSize::Inches7Point3,
            rotation: Rotation::Degrees90,
        };
        let first = store.page(small).expect("drawn");
        let again = store.page(small).expect("kept");
        assert!(Arc::ptr_eq(&first, &again));
        assert_eq!(
            first.panel.as_ref().map(|body| body.plain.len()),
            Some(192_000)
        );
        assert!(store.get().is_some());
        assert_eq!(draws.load(Ordering::SeqCst), 2);

        store.publish(Box::new(draw));
        let fresh = store.page(small).expect("drawn again");
        assert!(!Arc::ptr_eq(&first, &fresh));
        let canvas = store
            .page(Target::Canvas(Resolution::new(320, 200)))
            .expect("drawn");
        assert!(canvas.panel.is_none());
    }

    #[test]
    fn keeps_each_view_apart() {
        let default = Target::Panel {
            panel: PanelSize::Inches7Point3,
            rotation: Rotation::Degrees0,
        };
        let store = PageStore::new(default);
        let views: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let seen: Arc<Mutex<Vec<String>>> = Arc::clone(&views);
        store.publish_views(Box::new(move |target, view| {
            seen.lock().ok()?.push(String::from(view));
            draw(target)
        }));
        let week = store.view(default, "desk/week").expect("drawn");
        let today = store.view(default, "desk/today").expect("drawn");
        assert!(!Arc::ptr_eq(&week, &today));
        assert!(Arc::ptr_eq(
            &week,
            &store.view(default, "desk/week").expect("kept")
        ));
        assert_eq!(
            views.lock().map(|views| views.clone()).ok(),
            Some(vec![String::from("desk/week"), String::from("desk/today")])
        );
    }
}
