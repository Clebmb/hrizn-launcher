//! The launcher's plates: one flat, still surface per card, dialog, menu or sheet.
//!
//! Nothing moves behind the pages (there is no animated backdrop, and no blur sampled from one), so
//! a plate is a solid colour: the accent's dark base lifted a little in the dark look, white in the
//! light one. Menus sit a shade deeper than cards. A frame of the launcher is just the elements it
//! draws: no images to bake, cache, blur or free.
use gpui::{div, prelude::*, px, rgb, AnyElement, Div, Pixels, Size, Window};
use horizon_theme::glass::Glass;
use std::cell::Cell;
use std::rc::Rc;

/// How far below the top bar the pages' scroll area starts, so what scrolls up stops short of the
/// bar instead of running into it. The pages' heads hold that much less above their titles, so
/// nothing moves at rest.
/// The frame's one margin: the sidebar's distance from the window's edges, the gap under the
/// top bar, and the space a page head leaves above its title. 14 px, as the mockup has it.
pub const FRAME: f32 = 14.;
/// What the frame has already given a page above its content.
pub const CONTENT_INSET: f32 = FRAME;

/// A plate's kind: a card or panel, or a menu and popover above one.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Glass,
    Float,
}

thread_local! {
    /// The look's colours for the player's accent.
    static LOOK: Cell<Glass> = Cell::new(Glass::default_look());
    /// The light appearance: plates are white.
    static LIGHT: Cell<bool> = const { Cell::new(false) };
}

/// The player's accent changed: plates take its colour again.
pub fn set_look(look: Glass) {
    LOOK.set(look);
}

/// The appearance changed between light and dark.
pub fn set_light(light: bool) {
    LIGHT.set(light);
}

/// A plate's fill.
fn fill(kind: Kind) -> gpui::Hsla {
    if LIGHT.get() {
        return gpui::white();
    }
    let look = LOOK.get();
    let lift = if kind == Kind::Float { 0.02 } else { 0.05 };
    // `mix_oklab` weights its first colour: 1 - lift of the dark base, lift of white. The other
    // way round paints the plates white, and the look's white ink disappears into them.
    rgb(horizon_theme::glass::mix_oklab(
        look.base,
        0xffffff,
        1. - lift,
    ))
    .into()
}

/// Called as the launcher's pages start being built. Plates are plain elements now, so there is
/// nothing to retire; the hook stays for the callers.
pub fn begin_build(_window: &mut Window) {}

/// A plate's surface with `radius` corners: put it first inside a `relative` element, the content
/// after it.
pub fn backdrop(kind: Kind, radius: f32) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .rounded(px(radius))
        .bg(fill(kind))
        .into_any_element()
}

/// A `relative` div on a plate with `radius` corners.
pub fn plate(radius: f32) -> Div {
    div()
        .relative()
        .rounded(px(radius))
        .child(backdrop(Kind::Glass, radius))
}

/// The same for a menu or popover.
pub fn float(radius: f32) -> Div {
    div()
        .relative()
        .rounded(px(radius))
        .child(backdrop(Kind::Float, radius))
}

/// The tabbed panel's plate: one panel, round at 20.
pub fn tabbed(_strip: Rc<Cell<Option<Size<Pixels>>>>) -> impl IntoElement {
    backdrop(Kind::Glass, horizon_theme::glass::radius::PANEL)
}

/// A place-holder for the tab strip's measured size, which one panel no longer needs.
pub fn measure(_into: Rc<Cell<Option<Size<Pixels>>>>) -> impl IntoElement {
    div().absolute().top_0().left_0().size_full()
}
