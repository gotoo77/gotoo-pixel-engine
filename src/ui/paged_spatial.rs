use crate::{Framebuffer, Rect};

use super::experimental_spatial::{
    GridSpec, SpatialCard, SpatialInput, SpatialOutput, SpatialState,
    run_default_card_grid_styled,
};
use super::{UiStyleSheet, UiTheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PagedGridSpec {
    pub page_size: usize,
    pub grid: GridSpec,
    pub wrap_pages: bool,
}

impl Default for PagedGridSpec {
    fn default() -> Self {
        Self {
            page_size: 6,
            grid: GridSpec::default(),
            wrap_pages: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct PagedSpatialState {
    page: usize,
    spatial: SpatialState,
}

impl PagedSpatialState {
    pub const fn page(&self) -> usize {
        self.page
    }

    pub const fn spatial(&self) -> &SpatialState {
        &self.spatial
    }

    pub fn spatial_mut(&mut self) -> &mut SpatialState {
        &mut self.spatial
    }

    pub fn reset_page(&mut self) {
        self.set_page(0);
    }

    pub fn set_page(&mut self, page: usize) {
        if self.page != page {
            self.page = page;
            self.spatial = SpatialState::default();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PagedGridInput<'a> {
    pub spatial: SpatialInput<'a>,
    pub previous_page: bool,
    pub next_page: bool,
}

impl<'a> Default for PagedGridInput<'a> {
    fn default() -> Self {
        Self {
            spatial: SpatialInput::default(),
            previous_page: false,
            next_page: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PagedGridOutput {
    spatial: SpatialOutput,
    page: usize,
    page_count: usize,
    page_start: usize,
    page_end: usize,
    total_items: usize,
    page_changed: bool,
}

impl PagedGridOutput {
    pub const fn spatial(&self) -> &SpatialOutput {
        &self.spatial
    }

    pub fn into_spatial(self) -> SpatialOutput {
        self.spatial
    }

    pub const fn page(&self) -> usize {
        self.page
    }

    pub const fn page_count(&self) -> usize {
        self.page_count
    }

    pub const fn page_number(&self) -> usize {
        self.page + 1
    }

    pub const fn page_start(&self) -> usize {
        self.page_start
    }

    pub const fn page_end(&self) -> usize {
        self.page_end
    }

    pub const fn total_items(&self) -> usize {
        self.total_items
    }

    pub const fn page_changed(&self) -> bool {
        self.page_changed
    }

    pub const fn has_previous_page(&self) -> bool {
        self.page > 0
    }

    pub const fn has_next_page(&self) -> bool {
        self.page + 1 < self.page_count
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_default_paged_card_grid_styled(
    framebuffer: &mut Framebuffer,
    bounds: Rect,
    state: &mut PagedSpatialState,
    input: PagedGridInput<'_>,
    spec: PagedGridSpec,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    cards: &[SpatialCard<'_>],
) -> PagedGridOutput {
    let page_size = spec.page_size.max(1);
    let page_count = page_count(cards.len(), page_size);
    let previous_page = state.page;

    state.page = normalize_page(state.page, page_count);
    if input.previous_page && page_count > 1 {
        state.page = if state.page > 0 {
            state.page - 1
        } else if spec.wrap_pages {
            page_count - 1
        } else {
            state.page
        };
    }
    if input.next_page && page_count > 1 {
        state.page = if state.page + 1 < page_count {
            state.page + 1
        } else if spec.wrap_pages {
            0
        } else {
            state.page
        };
    }

    let page_changed = previous_page != state.page;
    if page_changed {
        state.spatial = SpatialState::default();
    }

    let page_start = state.page.saturating_mul(page_size).min(cards.len());
    let page_end = page_start.saturating_add(page_size).min(cards.len());
    let page_cards = &cards[page_start..page_end];

    let spatial = run_default_card_grid_styled(
        framebuffer,
        bounds,
        &mut state.spatial,
        input.spatial,
        spec.grid,
        theme,
        stylesheet,
        page_cards,
    );

    PagedGridOutput {
        spatial,
        page: state.page,
        page_count,
        page_start,
        page_end,
        total_items: cards.len(),
        page_changed,
    }
}

const fn page_count(item_count: usize, page_size: usize) -> usize {
    if item_count == 0 {
        1
    } else {
        item_count.div_ceil(page_size)
    }
}

const fn normalize_page(page: usize, page_count: usize) -> usize {
    if page >= page_count {
        page_count.saturating_sub(1)
    } else {
        page
    }
}

#[cfg(test)]
mod tests {
    use crate::{ActionId, Image};

    use super::super::experimental::UiId;
    use super::*;

    const ACTION: ActionId = ActionId::new("paged-grid.test");

    fn cards(count: usize) -> Vec<SpatialCard<'static>> {
        (0..count)
            .map(|_| SpatialCard {
                id: UiId::ROOT,
                title: "CARD",
                subtitle: "",
                image: None::<&'static Image>,
                action: ACTION,
            })
            .collect()
    }

    fn framebuffer() -> Framebuffer {
        Framebuffer::new(640, 360)
    }

    fn bounds() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 640,
            height: 360,
        }
    }

    fn spec() -> PagedGridSpec {
        PagedGridSpec {
            page_size: 6,
            grid: GridSpec {
                min_cell_width: 180,
                preferred_cell_height: 130,
                gap: 12,
                padding: 0,
            },
            wrap_pages: false,
        }
    }

    #[test]
    fn seven_cards_are_split_into_six_plus_one() {
        let mut framebuffer = framebuffer();
        let mut state = PagedSpatialState::default();
        let cards = cards(7);

        let first = run_default_paged_card_grid_styled(
            &mut framebuffer,
            bounds(),
            &mut state,
            PagedGridInput::default(),
            spec(),
            UiTheme::default(),
            UiStyleSheet::default(),
            &cards,
        );
        assert_eq!(first.page_number(), 1);
        assert_eq!(first.page_count(), 2);
        assert_eq!((first.page_start(), first.page_end()), (0, 6));
        assert_eq!(first.spatial().layouts().len(), 6);

        let second = run_default_paged_card_grid_styled(
            &mut framebuffer,
            bounds(),
            &mut state,
            PagedGridInput {
                next_page: true,
                ..PagedGridInput::default()
            },
            spec(),
            UiTheme::default(),
            UiStyleSheet::default(),
            &cards,
        );
        assert_eq!(second.page_number(), 2);
        assert_eq!((second.page_start(), second.page_end()), (6, 7));
        assert_eq!(second.spatial().layouts().len(), 1);
        assert!(second.page_changed());
    }

    #[test]
    fn page_does_not_wrap_when_disabled() {
        let mut framebuffer = framebuffer();
        let mut state = PagedSpatialState::default();
        state.set_page(1);
        let cards = cards(7);

        let output = run_default_paged_card_grid_styled(
            &mut framebuffer,
            bounds(),
            &mut state,
            PagedGridInput {
                next_page: true,
                ..PagedGridInput::default()
            },
            spec(),
            UiTheme::default(),
            UiStyleSheet::default(),
            &cards,
        );
        assert_eq!(output.page_number(), 2);
        assert!(!output.page_changed());
    }

    #[test]
    fn page_clamps_when_card_count_shrinks() {
        let mut framebuffer = framebuffer();
        let mut state = PagedSpatialState::default();
        state.set_page(2);
        let cards = cards(4);

        let output = run_default_paged_card_grid_styled(
            &mut framebuffer,
            bounds(),
            &mut state,
            PagedGridInput::default(),
            spec(),
            UiTheme::default(),
            UiStyleSheet::default(),
            &cards,
        );
        assert_eq!(output.page_number(), 1);
        assert_eq!(output.page_count(), 1);
        assert_eq!(output.spatial().layouts().len(), 4);
        assert!(output.page_changed());
    }

    #[test]
    fn empty_grid_still_reports_one_page() {
        let mut framebuffer = framebuffer();
        let mut state = PagedSpatialState::default();
        let cards = cards(0);

        let output = run_default_paged_card_grid_styled(
            &mut framebuffer,
            bounds(),
            &mut state,
            PagedGridInput::default(),
            spec(),
            UiTheme::default(),
            UiStyleSheet::default(),
            &cards,
        );
        assert_eq!(output.page_number(), 1);
        assert_eq!(output.page_count(), 1);
        assert_eq!(output.total_items(), 0);
    }
}
