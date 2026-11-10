//! The Settings page in the glass look. The sections live in one menu under the title: the page
//! draws only the chosen one, in its own scrolling area, so nothing has to be scrolled past to
//! reach the setting wanted. The rows inside are the mockup's `SettingRow`s
//! ([`glass_ui::flat_rows`]).
//!
//! Not in the mockup, kept in its patterns: the appearance tiles and the Home background, the
//! logs folder and the applications menu (rows under About's tiles), the AI clients, and the
//! hidden developer options. The mockup's Native mods switch is the native mods' build profile
//! here: they are always on.

use super::*;
use crate::glass_ui::{self as ui, white};
use horizon_theme::glass::{fill, ink, radius};

/// The menu's entries: name and icon.
const SECTIONS: [(&str, &str); 8] = [
    ("Account", "user"),
    ("Appearance", "color-picker"),
    ("Updates", "refresh"),
    ("Launcher", "computer"),
    ("Network", "globe-02"),
    ("Game", "game-controller-01"),
    ("AI", "ai-chat-02"),
    ("About", "information-circle"),
];

/// `HORIZON_LAUNCHER_SECTION=launcher`: the section the page starts on, for screenshots. A
/// script cannot click its way to one (synthetic clicks never reach the window), so this picks
/// it before the first frame. `account`, `appearance`, `updates`, `launcher`, `network`, `game`,
/// `ai`, `about` (a section's label, lowercased); anything else, or no variable, starts at the
/// top.
pub(super) fn section_from_env() -> Option<usize> {
    let name = std::env::var("HORIZON_LAUNCHER_SECTION").ok()?;
    SECTIONS
        .iter()
        .position(|(label, _)| label.to_ascii_lowercase().replace(' ', "-") == name)
}

/// What `settings_page` builds before it knows which look draws it.
pub(super) struct Built {
    pub install: Stateful<Div>,
    pub prune: Stateful<Div>,
    pub opening: Stateful<Div>,
    pub menu_entry: Stateful<Div>,
    pub horizon: String,
    pub horizon_action: Option<AnyElement>,
    pub launcher: String,
    pub launcher_action: Option<AnyElement>,
}

/// A section's card (`hzl-glass rounded-[20px] p-2`).
fn card() -> Div {
    crate::glass::plate(radius::PANEL)
        .flex()
        .flex_col()
        .p(px(8.))
}

/// An About tile (`rounded-[14px] bg-white/[0.045] p-5`): its lead, name, a line under it and
/// what it does. 180 px is its floor: below three across, the tiles wrap instead of squeezing their
/// action button and status line out of the card.
fn about_tile(lead: AnyElement, title: String, note: String, action: AnyElement) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w(px(180.))
        .rounded(px(radius::LARGE))
        .bg(white(0.045))
        .p(px(20.))
        .child(div().child(lead))
        .child(
            ui::text(15., FontWeight::SEMIBOLD, ink::FULL)
                .mt(px(16.))
                .child(title),
        )
        .child(
            ui::text(13., FontWeight::NORMAL, 0.6)
                .mt(px(2.))
                .truncate()
                .child(note),
        )
        .child(div().mt(px(16.)).flex().items_start().child(action))
}

fn note(s: &str) -> Option<AnyElement> {
    Some(div().child(tr(s).to_string()).into_any_element())
}

impl Launcher {
    pub(super) fn glass_settings(&mut self, b: Built, cx: &mut Context<Self>) -> Div {
        let active = self.settings_tab.min(SECTIONS.len() - 1);
        let menu = self.settings_sections_menu(active, cx);
        let body = self.glass_settings_section(active, b, cx);
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .child(ui::page_head_at(
                self.window_width,
                tr("Settings"),
                tr("Accounts, updates, and the launcher itself."),
            ))
            .child(menu)
            .child(
                div()
                    .id("settings-pane")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_y_scroll()
                    .child(body),
            )
    }

    /// The sections' menu under the title: the section shown, with its icon, and, open, the rest
    /// of them on a plate of their own. Picking one draws that section, and only it, below.
    fn settings_sections_menu(&mut self, active: usize, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let (label, icon) = SECTIONS[active];
        let trigger = keyboard(
            div()
                .id("settings-sections")
                .flex()
                .flex_none()
                .items_center()
                .gap(px(10.))
                .h(px(40.))
                .px(px(14.))
                .rounded(px(radius::CONTROL))
                .bg(white(fill::BUTTON))
                .cursor_pointer()
                .hover(|s| s.bg(white(fill::ROW_HOVER)))
                .font_family(horizon_theme::glass::FONT)
                .text_size(px(14.))
                .text_color(white(ink::FULL))
                .child(ui::icon(icon, 17., white(0.7)))
                .child(tr(label).to_string())
                .child(ui::icon("arrow-down-01", 16., white(0.5))),
            &t,
        )
        .on_click(cx.listener(|this, event: &ClickEvent, _, cx| {
            if this.closed_by_this_press(event) {
                return;
            }
            this.settings_menu = !this.settings_menu;
            cx.notify();
        }));
        let mut row = div().relative().mb(px(12.)).child(trigger);
        if self.settings_menu {
            let mut list = crate::glass::plate(radius::PANEL)
                .absolute()
                .top(px(46.))
                .left_0()
                .w(px(240.))
                .p(px(6.))
                .flex()
                .flex_col()
                .occlude()
                .on_mouse_down_out(cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.settings_menu = false;
                    this.menu_closed_at = Some(event.position);
                    cx.notify();
                }));
            for (i, &(label, icon)) in SECTIONS.iter().enumerate() {
                let on = i == active;
                list = list.child(
                    div()
                        .id(SharedString::from(format!("settings-section-{i}")))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .h(px(38.))
                        .px(px(10.))
                        .rounded(px(radius::ROW))
                        .cursor_pointer()
                        .font_family(horizon_theme::glass::FONT)
                        .text_size(px(14.))
                        .when(on, |d| {
                            d.bg(white(fill::ACTIVE))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(white(ink::FULL))
                        })
                        .when(!on, |d| {
                            d.text_color(white(0.7))
                                .hover(|s| s.bg(white(0.05)).text_color(white(ink::FULL)))
                        })
                        .child(ui::icon(
                            icon,
                            16.,
                            if on {
                                ui::color(ui::look().accent_soft)
                            } else {
                                white(0.6)
                            },
                        ))
                        .child(tr(label).to_string())
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.settings_tab = i;
                            this.settings_menu = false;
                            cx.notify();
                        })),
                );
            }
            row = row.child(gpui::deferred(list).with_priority(1));
        }
        row
    }

    /// The one section the menu picked. Each is a card, as the mockup's sections were, in the
    /// order of [`SECTIONS`].
    fn glass_settings_section(&mut self, active: usize, b: Built, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let n = 0;
        match active {
            0 => card().child(self.glass_account(cx)),
            1 => card()
                .child(self.appearance_mode_row(cx))
                .child(self.glass_accent_row(cx))
                .child(self.reset_look_row(cx)),
            2 => card()
                .child(index_row(
                    0,
                    tr("Install new versions automatically"),
                    note("Your selected version only changes if you were playing the newest one."),
                    Some(b.install.into_any_element()),
                    &t,
                ))
                .child(index_row(
                    0,
                    tr("Delete old versions automatically"),
                    Some(
                        div()
                            .child(trf(
                                "Keeps the {count} newest, the one in use and the ones your instances use. Deleted versions install again from the Versions page.",
                                &[("count", &self.settings.keep_newest().to_string())],
                            ))
                            .into_any_element(),
                    ),
                    Some(b.prune.into_any_element()),
                    &t,
                )),
            3 => card()
                .child(self.language_row(cx))
                .child(index_row(
                    0,
                    tr("Show the opening"),
                    note("The wordmark, while the release list loads."),
                    Some(b.opening.into_any_element()),
                    &t,
                ))
                .child(self.show_versions_row(cx))
                .child(self.show_patch_notes_row(cx))
                .child(self.while_playing_row(n, cx)),
            4 => card().child(self.proxy_row(n, cx)),
            5 => card()
                .child(self.game_architecture_row(n, cx))
                .child(self.game_args_row(n + 1, cx))
                .child(self.mods_auto_update_row(cx))
                .child(self.native_mods_row(cx)),
            6 => card()
                .child(self.no_ai_row(n, cx))
                .children(self.agents_section(cx)),
            _ => card()
                .child(self.about_tiles(b.horizon, b.horizon_action, b.launcher, b.launcher_action))
                .child(index_row(
                    0,
                    tr("Logs"),
                    Some({
                        let s = format!(
                            "{}  ·  {}",
                            short(&backend::logs_dir()),
                            tr("crash reports beside it")
                        );
                        div()
                            .font_family(theme::MONO)
                            .text_size(px(12.))
                            .child(s)
                            .into_any_element()
                    }),
                    Some(
                        button("open-logs", "Open", Kind::Secondary, &t)
                            .on_click(|_, _, cx| {
                                let dir = backend::logs_dir();
                                let _ = std::fs::create_dir_all(&dir);
                                crate::glass_toast::open(cx, &dir);
                            })
                            .into_any_element(),
                    ),
                    &t,
                ))
                .when(cfg!(target_os = "linux"), |d| {
                    d.child(index_row(
                        0,
                        tr("Applications menu"),
                        note(if self.settings.desktop_entry() {
                            "Horizon and its icon are in your desktop's menu."
                        } else {
                            "Not in your desktop's menu."
                        }),
                        Some(b.menu_entry.into_any_element()),
                        &t,
                    ))
                })
                .children(self.dev_options_section(cx)),
        }
    }

    /// The appearance's three ways, as plates that show what they do: Dark, Light and System,
    /// which follows the computer. The plate of the one in use carries the accent's edge.
    fn appearance_mode_row(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let chosen = self.settings.appearance;
        let accent = ui::color(ui::look().accent_soft);
        let bar = |width: f32, colour: Hsla| div().w(px(width)).h(px(4.)).rounded(px(2.)).bg(colour);
        let plate = |id: &'static str, on: bool| {
            keyboard(
                div()
                    .id(id)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(112.))
                    .gap(px(8.))
                    .p(px(8.))
                    .rounded(px(radius::LARGE))
                    .bg(if on { white(0.06) } else { white(0.02) })
                    .border_1()
                    .border_color(if on { accent } else { white(0.07) })
                    .cursor_pointer()
                    .when(!on, |d| d.hover(|s| s.bg(white(0.06)))),
                &t,
            )
        };
        let preview = |bg: Hsla, ink: Hsla| {
            div()
                .flex()
                .flex_col()
                .justify_end()
                .gap(px(4.))
                .w_full()
                .h(px(56.))
                .p(px(9.))
                .rounded(px(radius::CONTROL))
                .bg(bg)
                .child(bar(28., ink))
                .child(bar(18., ink))
                .into_any_element()
        };
        let mut split = div()
            .flex()
            .w_full()
            .h(px(56.))
            .rounded(px(radius::CONTROL))
            .overflow_hidden();
        for half in [gpui::rgb(0x17141f), gpui::rgb(0xefedf5)] {
            split = split.child(div().flex_1().h_full().p(px(9.)).bg(half).flex().flex_col().justify_end().gap(px(4.)).child(bar(20., white(0.55))).child(bar(12., white(0.3))));
        }
        let dark = plate("appearance-dark", chosen == backend::Appearance::Dark)
            .child(preview(gpui::rgb(0x17141f).into(), white(0.85)))
            .child(ui::text(13., FontWeight::MEDIUM, ink::FULL).child(tr("Dark")))
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                this.settings.appearance = backend::Appearance::Dark;
                this.save_settings();
                crate::theme::unpin();
                titlebar::apply_appearance(window, backend::Appearance::Dark);
                this.theme = this.settings_theme(window);
                cx.notify();
            }));
        let light = plate("appearance-light", chosen == backend::Appearance::Light)
            .child(preview(gpui::rgb(0xefedf5).into(), gpui::rgb(0x17141f).into()))
            .child(ui::text(13., FontWeight::MEDIUM, ink::FULL).child(tr("Light")))
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                this.settings.appearance = backend::Appearance::Light;
                this.save_settings();
                crate::theme::unpin();
                titlebar::apply_appearance(window, backend::Appearance::Light);
                this.theme = this.settings_theme(window);
                cx.notify();
            }));
        let system = plate("appearance-system", chosen == backend::Appearance::System)
            .child(split)
            .child(ui::text(13., FontWeight::MEDIUM, ink::FULL).child(tr("System")))
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                this.settings.appearance = backend::Appearance::System;
                this.save_settings();
                crate::theme::unpin();
                titlebar::apply_appearance(window, backend::Appearance::System);
                this.theme = this.settings_theme(window);
                cx.notify();
            }));
        ui::setting_row(
            tr("Appearance"),
            note("Dark by default. System follows your computer's light or dark."),
            Some(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .w_full()
                    .child(dark)
                    .child(light)
                    .child(system)
                    .into_any_element(),
            ),
        )
    }

    /// The Launcher card's switch for the Versions page in the menu (`Page::in_top_bar`).
    fn show_versions_row(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let on = self.settings.versions_tab();
        ui::setting_row(
            tr("Show Versions Tab"),
            note("The Versions page in the menu under the launcher's name."),
            Some(
                crate::switch("show-versions-tab", on, &t)
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.settings.show_versions = Some(!this.settings.versions_tab());
                        this.save_settings();
                        cx.notify();
                    }))
                    .into_any_element(),
            ),
        )
    }

    /// The Launcher card's switch for the Patch notes page in the menu.
    fn show_patch_notes_row(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let on = self.settings.patch_notes_tab();
        ui::setting_row(
            tr("Show Patch Notes Tab"),
            note("The Patch notes page in the menu under the launcher's name."),
            Some(
                crate::switch("show-patch-notes-tab", on, &t)
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.settings.show_patch_notes = Some(!this.settings.patch_notes_tab());
                        this.save_settings();
                        cx.notify();
                    }))
                    .into_any_element(),
            ),
        )
    }

    /// The Game card's switch for registry mods' automatic updates (`horizon::auto`).
    fn mods_auto_update_row(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let on = self.settings.auto_updates_mods();
        ui::setting_row(
            tr("Update registry mods automatically"),
            Some(
                div()
                    .child(tr(
                        "At start and before you play, what you installed from Horizon's registry moves to its newest version for this game. Never inside a modpack, and an update that would remove blocks or items waits for you.",
                    ))
                    .into_any_element(),
            ),
            Some(
                crate::switch("mods-auto-update", on, &t)
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.settings.mods_auto_update = Some(!this.settings.auto_updates_mods());
                        this.save_settings();
                        cx.notify();
                    }))
                    .into_any_element(),
            ),
        )
    }

    /// The Game card's Native mods row. The mockup has an on/off switch; native mods are always
    /// on here, so the row picks the profile they build with, as the Mods page does.
    fn native_mods_row(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let profile = native_mods::Profile::chosen(&self.settings);
        let picker = segments(
            "settings-native-profile",
            &[
                (native_mods::Profile::Performance, "Performance build"),
                (native_mods::Profile::Dev, "Dev build"),
                (native_mods::Profile::Release, "Release build"),
            ],
            profile,
            &t,
            {
                let this = cx.entity().downgrade();
                move |choice, _, cx| {
                    let _ = this.update(cx, |this, cx| {
                        this.settings.native_profile = Some(choice);
                        this.settings.native_release = false;
                        this.native_done = None;
                        this.save_settings();
                        cx.notify();
                    });
                }
            },
        );
        let said = match profile {
            native_mods::Profile::Performance => tr("The everyday build for playing with mods"),
            native_mods::Profile::Dev => tr(
                "For developing a mod: about 6× faster to rebuild a mod edit, game logic up to about twice as slow",
            ),
            native_mods::Profile::Release => {
                tr("Release build: slower to build, slightly faster game")
            }
        };
        ui::setting_row(
            tr("Native mods"),
            Some(
                div()
                    .child(format!(
                        "{} {said}.",
                        tr("Experimental. Builds the native mods you trust into a game of their own.")
                    ))
                    .into_any_element(),
            ),
            Some(picker.into_any_element()),
        )
    }

    /// About's three tiles (the mockup's `AboutTile`s): Horizon and its update, the launcher and
    /// its own, and the data folder.
    fn about_tiles(
        &mut self,
        horizon: String,
        horizon_action: Option<AnyElement>,
        launcher: String,
        launcher_action: Option<AnyElement>,
    ) -> Div {
        let t = self.theme;
        let installed = self
            .settings
            .selected
            .clone()
            .map(|tag| match &self.releases {
                Load::Ready(list) => list
                    .iter()
                    .find(|r| r.tag == tag)
                    .map_or(tag.clone(), |r| update_name(r).to_owned()),
                _ => tag,
            });
        let tone = if self.updates.horizon.is_some() {
            ui::Tone::Accent(ui::look().accent_soft)
        } else if self.update_error.is_some() {
            ui::Tone::Warn
        } else if self.checked.is_some() {
            ui::Tone::Live
        } else {
            ui::Tone::Busy
        };
        let ink = match tone {
            ui::Tone::Live => ui::live(),
            ui::Tone::Warn => ui::warn(),
            _ => white(0.7),
        };
        let status = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .text_size(px(13.))
            .text_color(ink)
            .child(ui::dot(tone))
            .child(div().min_w_0().truncate().child(horizon));
        let square = |icon: &'static str| {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(42.))
                .rounded(px(radius::CONTROL))
                .bg(white(0.08))
                .child(ui::icon(icon, 21., white(0.8)))
                .into_any_element()
        };
        let open = ui::quiet("about-open-data", tr("Open"), 36., &t)
            .on_click(|_, _, cx| crate::glass_toast::open(cx, &backend::data_dir()))
            .into_any_element();
        div()
            .flex()
            .flex_wrap()
            .gap(px(8.))
            .child(about_tile(
                self.horizon_mark(42.),
                "Horizon".into(),
                installed.unwrap_or_else(|| tr("Nightly").to_string()),
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(12.))
                    .child(status)
                    .children(horizon_action)
                    .into_any_element(),
            ))
            .child(about_tile(
                square("computer"),
                tr("Launcher").to_string(),
                launcher,
                div().children(launcher_action).into_any_element(),
            ))
            .child(about_tile(
                square("folder-01"),
                tr("Data folder").to_string(),
                short(&backend::data_dir()),
                open,
            ))
    }
}
