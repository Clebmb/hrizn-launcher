//! Settings' accent colour and Home background rows (`horizon_launcher::themes`).
use super::*;
use crate::glass_ui as ui;
use crate::servers_page::Field;
use gpui::{
    canvas, fill, point, size, Bounds, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point,
};
use horizon_launcher::themes::{self, Accent};
use std::path::PathBuf;

/// Each accent's swatch, in points.
const SWATCH: f32 = 26.;
/// The hue strip beside the swatches, in points: 0 to 360 degrees across it.
const STRIP: f32 = 120.;

impl Launcher {
    /// The theme Settings ask for, under the window's own light or dark: the player's look,
    /// under the launcher theme's sky if they picked one.
    pub(super) fn settings_theme(&self, window: &Window) -> Theme {
        let theme = Theme::for_appearance(
            self.settings.appearance,
            self.settings.accent,
            window.appearance(),
        );
        match themes::theme(self.settings.theme.as_deref()) {
            Some(picked) => theme.with_sky(picked.sky),
            None => theme,
        }
    }

    /// The picture behind Home, if the player keeps one.
    pub(super) fn home_background(&self) -> Option<PathBuf> {
        themes::background(&backend::data_dir(), self.settings.background.as_deref())
    }

    pub(super) fn accent_row(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let mut swatches = div().flex().gap(px(6.));
        for accent in Accent::ALL {
            let face = Theme::for_appearance(
                if t.dark {
                    backend::Appearance::Dark
                } else {
                    backend::Appearance::Light
                },
                accent,
                gpui::WindowAppearance::Dark,
            )
            .face;
            let chosen = self.settings.accent == accent;
            swatches = swatches.child(
                keyboard(
                    div().id(SharedString::from(format!("accent-{}", accent.name()))),
                    &t,
                )
                .size(px(SWATCH))
                .bg(face)
                .border_2()
                .border_color(if chosen { t.ink } else { t.outline })
                .cursor_pointer()
                .when(chosen, |d| {
                    d.flex()
                        .items_center()
                        .justify_center()
                        .child(div().size(px(8.)).bg(t.on_face))
                })
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    this.settings.accent = accent;
                    this.save_settings();
                    this.theme = this.settings_theme(window);
                    cx.notify();
                })),
            );
        }
        index_row(
            0,
            tr("Accent colour"),
            Some(
                div()
                    .child(tr(
                        "Horizon's violet by default. Every colour keeps text as easy to read.",
                    ))
                    .into_any_element(),
            ),
            Some(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(swatches)
                    .child(self.hue_strip(&t, cx))
                    .into_any_element(),
            ),
            &t,
        )
    }

    /// The glass look's accent row (the mockup's): its seven swatches, the any-colour swatch
    /// (a click shows the hue strip under them, the browser's colour picker in the mockup) and
    /// the hex field, which takes the colour as soon as it is one.
    pub(super) fn glass_accent_row(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        self.sync_accent_hex(cx);
        let current = self.settings.accent;
        let custom = !Accent::ALL.contains(&current);
        let swatch = |id: SharedString, on: bool| {
            keyboard(
                div()
                    .id(id)
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(36.))
                    .rounded(px(10.))
                    .cursor_pointer()
                    .when(on, |d| d.bg(ui::white(0.18)))
                    .when(!on, |d| d.hover(|s| s.bg(ui::white(0.08)))),
                &t,
            )
        };
        let chip = |rgb: u32| div().size(px(24.)).rounded(px(7.)).bg(gpui::rgb(rgb));
        let mut row = div().flex().flex_wrap().items_center().gap(px(4.));
        for (accent, rgb) in Accent::ALL.into_iter().zip(themes::SWATCHES) {
            row = row.child(
                swatch(
                    SharedString::from(format!("accent-{}", accent.name())),
                    current == accent,
                )
                .child(chip(rgb))
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.settings.accent = accent;
                    this.save_settings();
                    this.restyle = true;
                    cx.notify();
                })),
            );
        }
        row = row
            .child(
                swatch("accent-any".into(), custom)
                    .child(if custom {
                        chip(current.rgb()).into_any_element()
                    } else {
                        ui::icon("color-picker", 20., ui::white(0.7)).into_any_element()
                    })
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.accent_strip = !this.accent_strip;
                        cx.notify();
                    })),
            )
            .child(self.field(Field::AccentHex, "#6b4bff", 96., cx).ml(px(4.)));
        let control = div()
            .flex()
            .flex_col()
            .items_end()
            .gap(px(10.))
            .child(row)
            .when(self.accent_strip, |d| {
                d.child(
                    self.hue_strip(&t, cx)
                        .w(px(212.))
                        .h(px(24.))
                        .rounded(px(6.))
                        .overflow_hidden(),
                )
            });
        ui::setting_row(
            tr("Accent colour"),
            Some(
                div()
                    .child(tr(
                        "Colours the buttons and the switches. Pick one, or type any hex colour.",
                    ))
                    .into_any_element(),
            ),
            Some(control.into_any_element()),
        )
    }

    /// Reset to defaults: the card's options go back to how Horizon ships (the mockup's
    /// accent, language, while playing and opening; here the appearance and Home's
    /// background too, being in the same card).
    pub(super) fn reset_look_row(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        ui::setting_row(
            tr("Reset the launcher's look"),
            Some(
                div()
                    .child(tr(
                        "Accent colour, language and the options above go back to how Horizon ships.",
                    ))
                    .into_any_element(),
            ),
            Some(
                ui::quiet_icon(
                    "look-reset",
                    Some("refresh"),
                    tr("Reset to defaults"),
                    40.,
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                    let s = &mut this.settings;
                    s.appearance = Default::default();
                    s.accent = Accent::VIOLET;
                    s.theme = None;
                    s.background = None;
                    s.language = None;
                    s.while_playing = Default::default();
                    s.skip_opening = false;
                    this.save_settings();
                    i18n::set(this.settings.language());
                    crate::theme::unpin();
                    titlebar::apply_appearance(window, this.settings.appearance);
                    this.accent_strip = false;
                    this.theme = this.settings_theme(window);
                    crate::glass_toast::say(tr("Launcher settings are back to their defaults"));
                    cx.notify();
                }))
                .into_any_element(),
            ),
        )
    }

    /// The hex field shows the accent, except while typed in: then each whole colour typed is
    /// the accent at once, as in the mockup.
    fn sync_accent_hex(&mut self, cx: &mut Context<Self>) {
        let shown = self.settings.accent.hex();
        if !self.servers.has_focus(Field::AccentHex) {
            if self.servers.value(Field::AccentHex) != shown {
                self.servers.text(Field::AccentHex).set(shown);
            }
            return;
        }
        let typed = self.servers.value(Field::AccentHex).trim();
        let hex = format!("#{}", typed.trim_start_matches('#'));
        if let Some(accent) = Accent::parse(&hex).filter(|a| *a != self.settings.accent) {
            self.settings.accent = accent;
            self.save_settings();
            self.restyle = true;
            cx.notify();
        }
    }

    /// A strip of every hue: a click or a drag picks any of them, the swatches being shortcuts.
    fn hue_strip(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let current = self.settings.accent.hue();
        let area = self.hue_area.clone();
        let ink = t.ink;
        let outline = t.outline;
        // Hues per painted quad: 60 of them, fine enough at this width.
        const STEPS: usize = 60;
        div()
            .relative()
            .flex_none()
            .w(px(STRIP))
            .h(px(SWATCH))
            .border_2()
            .border_color(outline)
            .cursor_pointer()
            .child(
                canvas(
                    move |bounds, _, _| {
                        area.set(Some(bounds));
                        bounds
                    },
                    move |bounds, _, window, _| {
                        let width: f32 = bounds.size.width.into();
                        let step = width / STEPS as f32;
                        for i in 0..STEPS {
                            let left = px(step * i as f32);
                            let colour: Hsla =
                                gpui::rgb(themes::hue_colour(i as f32 * 360. / STEPS as f32))
                                    .into();
                            window.paint_quad(fill(
                                Bounds::new(
                                    point(bounds.left() + left, bounds.top()),
                                    size(px(step), bounds.size.height),
                                ),
                                colour,
                            ));
                        }
                        if let Some(hue) = current {
                            let at = px(width * hue / 360.);
                            window.paint_quad(fill(
                                Bounds::new(
                                    point(bounds.left() + at - px(1.5), bounds.top()),
                                    size(px(3.), bounds.size.height),
                                ),
                                ink,
                            ));
                        }
                    },
                )
                .size_full(),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, window, cx| {
                    this.pick_hue(ev.position, window, cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, window, cx| {
                if ev.pressed_button == Some(MouseButton::Left) {
                    this.pick_hue(ev.position, window, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.finish_hue();
                    cx.notify();
                }),
            )
            // `on_mouse_up` fires only while the strip is hovered, so a drag released off it would
            // keep its accent in memory and never write it.
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.finish_hue();
                    cx.notify();
                }),
            )
    }

    /// A release ends the drag and writes `settings.json` once.
    fn finish_hue(&mut self) {
        if self.hue_dragging {
            self.hue_dragging = false;
            self.save_settings();
        }
    }

    /// The hue under the pointer, as the accent. Saving waits until the mouse comes up, so a drag
    /// writes `settings.json` once. A move that stays on the same degree changes nothing.
    fn pick_hue(&mut self, at: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(area) = self.hue_area.get() else {
            return;
        };
        let width: f32 = area.size.width.into();
        if width <= 0. {
            return;
        }
        let across: f32 = (at.x - area.origin.x).into();
        let accent = Accent::custom(((across / width).clamp(0., 1.) * 360.).round());
        self.hue_dragging = true;
        if accent == self.settings.accent {
            return;
        }
        self.settings.accent = accent;
        self.theme = self.settings_theme(window);
        cx.notify();
    }

}
