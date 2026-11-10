//! The Patch notes page: HRIZN's own releases, one glass card each. There is one so far — the
//! first release — and the classic look shows the same note (`notes_page` in main.rs).

use super::*;
use crate::glass_ui::{self as ui};
use horizon_theme::glass::{ink, radius};

/// HRIZN's first release note: its title and body, for both looks.
pub(super) fn initial_release() -> (&'static str, &'static str) {
    (
        tr("HRIZN 0.1.0 - INITIAL RELEASE"),
        tr("HRIZN is a lighter, quieter way to start Minecraft: Rust Edition. It does the job a launcher has to do — install the version you want, keep it up to date, launch it — and then steps aside. Everything else is measured against one question: does this help you get into a world sooner, or does it stand between you and the game?\n\nThis first release is deliberately small. Downloads, versions and worlds work as they did before, so an existing setup carries over untouched. What changed is the surface: shorter menus, calmer screens, fewer places to get lost in. Nothing needs an account to open, and nothing you do here is collected, profiled or sent anywhere you did not ask for.\n\nHRIZN is not an attempt to replace Horizon for everyone. It exists because some players want a launcher that starts quickly, stays light, and asks very little of the computer it runs on. If that is what you were looking for, this is the beginning — and we intend to keep it small on purpose.\n\nThank you for using HRIZN Launcher. Whether you have followed along since the first build or have only just found us, thank you for giving it a try, and for telling us what still feels too heavy."),
    )
}

impl Launcher {
    pub(super) fn glass_notes(&mut self, _cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let (title, body) = initial_release();
        let page = div().flex().flex_col().child(ui::page_head_at(
            self.window_width,
            tr("Patch notes"),
            tr("What changed in each release, newest first."),
        ));
        page.child(
            crate::glass::plate(radius::PANEL)
                .flex()
                .flex_col()
                .gap(px(12.))
                .p(px(24.))
                .child(
                    ui::text(16., FontWeight::SEMIBOLD, ink::FULL)
                        .line_height(px(24.))
                        .child(title),
                )
                .child(div().child(markdown::render(body, &t))),
        )
    }
}
