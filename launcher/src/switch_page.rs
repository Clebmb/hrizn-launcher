//! Who plays: the top bar's quick switch (a menu under the account chip) and Settings' offline
//! names (`horizon_launcher::account::Accounts::offline`).
use super::servers_page::Field;
use super::*;
use horizon_launcher::account::Player;

impl Launcher {
    /// The chip's two lines: the name that plays, and how.
    pub(super) fn who_plays(&self) -> (String, &'static str) {
        match self.accounts.player() {
            Player::Microsoft(_) => (
                self.accounts
                    .selected()
                    .map_or_else(String::new, |a| a.name.clone()),
                tr("Microsoft account"),
            ),
            Player::Offline(name) => (name, tr("Offline")),
            Player::GameName => (
                self.offline_name.clone().unwrap_or_else(|| "Player".into()),
                tr("Offline"),
            ),
        }
    }

    pub(super) fn play_as(&mut self, who: Player) {
        let mut accounts = Accounts::load();
        let done = accounts
            .play_as(account::store(), who)
            .and_then(|()| accounts.save());
        match done {
            Ok(()) => self.accounts = accounts,
            Err(e) => self.sign_in_state = Some(SignInState::Done(Err(e))),
        }
        self.account_menu = false;
    }

    /// Everyone the player can play as, with how each signs in.
    pub(super) fn players(&self) -> Vec<(Player, String, &'static str)> {
        let mut out: Vec<_> = self
            .accounts
            .accounts
            .iter()
            .map(|a| {
                (
                    Player::Microsoft(a.uuid.clone()),
                    a.name.clone(),
                    tr("Microsoft"),
                )
            })
            .collect();
        out.extend(
            self.accounts
                .offline
                .iter()
                .map(|n| (Player::Offline(n.clone()), n.clone(), tr("Offline"))),
        );
        // The name the game made up is an account once it exists, or while it is the one
        // playing: until the game has run there is no such profile, not one called "Player".
        if self.offline_name.is_some() || self.accounts.player() == Player::GameName {
            out.push((
                Player::GameName,
                self.offline_name.clone().unwrap_or_else(|| "Player".into()),
                tr("Offline, the game's name"),
            ));
        }
        out
    }

    /// The menu under the account chip, while open.
    pub(super) fn account_menu(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        if !self.account_menu {
            return None;
        }
        let t = self.theme;
        let now = self.accounts.player();
        let mut menu = div()
            .id("account-menu")
            .absolute()
            .top(px(46.))
            .right_0()
            .w(px(280.))
            .p(px(6.))
            .border_2()
            .border_color(t.outline)
            .bg(t.surface)
            .flex()
            .flex_col()
            .occlude()
            .on_mouse_down_out(cx.listener(|this, event: &MouseDownEvent, _, cx| {
                this.account_menu = false;
                this.menu_closed_at = Some(event.position);
                cx.notify();
            }));
        for (i, (who, name, how)) in self.players().into_iter().enumerate() {
            let chosen = who == now;
            menu = menu.child(
                div()
                    .id(SharedString::from(format!("account-pick-{i}")))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.))
                    .h(px(46.))
                    .px(px(10.))
                    .rounded(px(theme::RADIUS))
                    .cursor_pointer()
                    .hover(move |s| s.bg(t.plate_soft))
                    .child(self.profile_heads.draw(
                        match &who {
                            Player::Microsoft(uuid) => Some(uuid),
                            _ => None,
                        },
                        24.,
                        self.scale,
                        &t,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(text(theme::LABEL).truncate().text_color(t.ink).child(name))
                            .child(text(theme::CAPTION).text_color(t.muted).child(how)),
                    )
                    .when(chosen, |d| {
                        d.child(div().size(px(6.)).rounded_full().bg(t.accent))
                    })
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.play_as(who.clone());
                        cx.notify();
                    })),
            );
        }
        menu = menu.child(div().my(px(6.)).h(px(1.)).bg(t.rule)).child(
            div()
                .id("account-manage")
                .flex()
                .items_center()
                .h(px(32.))
                .px(px(10.))
                .rounded(px(theme::RADIUS))
                .text_size(px(theme::LABEL.px))
                .text_color(t.muted)
                .cursor_pointer()
                .hover(move |s| s.bg(t.plate_soft).text_color(t.ink))
                .child(tr("Manage accounts"))
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.account_menu = false;
                    this.go(Page::Settings);
                    // The Account card, not whichever section was open last.
                    this.settings_tab = 0;
                    this.settings_menu = false;
                    cx.notify();
                })),
        );
        Some(gpui::deferred(menu).with_priority(1))
    }

    fn add_offline_name(&mut self) {
        let name = self.servers.value(Field::OfflineName).trim().to_string();
        if name.is_empty() {
            return;
        }
        self.play_as(Player::Offline(name));
        if !matches!(self.sign_in_state, Some(SignInState::Done(Err(_)))) {
            self.servers.text(Field::OfflineName).set(String::new());
        }
    }

    /// Asks before deleting an account: the card above the lists holds the question until the
    /// player answers it.
    pub(super) fn ask_account_delete(&mut self, who: AccountDelete) {
        self.account_delete = Some(who);
    }

    /// The account the question names, as the lists show it.
    pub(super) fn account_delete_name(&self) -> Option<String> {
        match self.account_delete.as_ref()? {
            AccountDelete::Offline(name) => Some(name.clone()),
            AccountDelete::Microsoft(uuid) => Some(
                self.accounts
                    .accounts
                    .iter()
                    .find(|a| &a.uuid == uuid)
                    .map_or_else(|| uuid.clone(), |a| a.name.clone()),
            ),
        }
    }

    /// Deletes the account the question names: an offline name with its skin and look, or a
    /// Microsoft account with its secrets. The worlds either one played in stay.
    pub(super) fn confirm_account_delete(&mut self, cx: &mut Context<Self>) {
        let Some(who) = self.account_delete.take() else {
            return;
        };
        match who {
            AccountDelete::Offline(name) => self.forget_offline(&name),
            AccountDelete::Microsoft(uuid) => self.sign_out_account(uuid),
        }
        cx.notify();
    }

    /// Signs `uuid` out here: its secrets and its sign-in go, its worlds stay.
    fn sign_out_account(&mut self, uuid: String) {
        let Some(generation) = self.begin_auth_job(&uuid) else {
            return;
        };
        let (tx, uuid) = (self.tx.clone(), uuid);
        std::thread::spawn(move || {
            let mut accounts = Accounts::load();
            let _ = tx.send(Msg::AccountProgress(
                uuid.clone(),
                generation,
                account::session::Phase::SigningOut,
            ));
            let result = accounts
                .remove(account::store(), &uuid)
                .and_then(|()| accounts.save())
                .map(|()| accounts.has_owner(account::store()));
            let _ = tx.send(Msg::AccountRemoved(uuid, generation, result));
        });
    }

    /// The question's two answers, for the classic look's row.
    pub(super) fn account_delete_actions(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        div()
            .flex()
            .gap(px(4.))
            .child(
                button("account-delete-yes", "Delete", Kind::Secondary, &t).on_click(
                    cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.confirm_account_delete(cx);
                    }),
                ),
            )
            .child(button("account-delete-no", "Cancel", Kind::Ghost, &t).on_click(
                cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.account_delete = None;
                    cx.notify();
                }),
            ))
            .into_any_element()
    }

    /// Forgets an offline name and its skin.
    pub(super) fn forget_offline(&mut self, name: &str) {
        let mut accounts = Accounts::load();
        if accounts
            .remove_offline(name, &horizon_launcher::skins::dir())
            .is_ok()
            && accounts.save().is_ok()
        {
            self.accounts = accounts;
        }
    }

    /// Enter in the offline name field.
    pub(super) fn offline_name_enter(&mut self) {
        self.add_offline_name();
    }

    /// Turns an offline name's chip (or the game's own name) into a field holding its name, so
    /// it can be edited where it is.
    pub(super) fn begin_offline_rename(
        &mut self,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.offline_rename = Some(name.to_string());
        self.servers
            .text(Field::OfflineRename)
            .set(name.to_string());
        self.servers.set_field_invalid(Field::OfflineRename, false);
        let handle = self.focus_handle(Field::OfflineRename, cx);
        window.focus(&handle, cx);
    }

    /// Leaves the name as it was.
    pub(super) fn cancel_offline_rename(&mut self, cx: &mut Context<Self>) {
        self.offline_rename = None;
        self.servers.text(Field::OfflineRename).set(String::new());
        self.servers.set_field_invalid(Field::OfflineRename, false);
        cx.notify();
    }

    /// Keeps the edited name. Its skin and cape files, its guest id (so its progress and servers
    /// stay its own) and its launcher picture move with it.
    pub(super) fn commit_offline_rename(&mut self) {
        let Some(old) = self.offline_rename.clone() else {
            return;
        };
        let new = self.servers.value(Field::OfflineRename).trim().to_string();
        if !account::valid_offline_name(&new) {
            self.servers.set_field_invalid(Field::OfflineRename, true);
            crate::glass_toast::say(tr(
                "3 to 16 letters, numbers or underscores. Online servers don't take offline \
                 names.",
            ));
            return;
        }
        let from = faces::key(&Player::Offline(old.clone()), &old);
        let mut accounts = Accounts::load();
        if accounts.offline.iter().any(|n| n.eq_ignore_ascii_case(&old)) {
            let renamed = accounts
                .rename_offline(&old, &new, &horizon_launcher::skins::dir())
                .and_then(|kept| accounts.save().map(|()| kept));
            match renamed {
                Ok(kept) => {
                    if let Some(from) = from.as_deref() {
                        faces::rename(&from.to_string(), &self.offline_key(&kept));
                    }
                    self.accounts = accounts;
                }
                Err(e) => {
                    self.servers.set_field_invalid(Field::OfflineRename, true);
                    crate::glass_toast::say(e);
                    return;
                }
            }
        } else {
            let Some(made_up) = self.offline_name.clone() else {
                return;
            };
            if let (Some(from), Some(to)) = (
                faces::key(&Player::GameName, &made_up),
                faces::key(&Player::GameName, &new),
            ) {
                faces::rename(&from, &to);
            }
            self.offline_name = Some(new);
        }
        if let Some(from) = from {
            self.profile_heads.set_custom(&from, None);
        }
        self.faces = faces::stored();
        self.load_faces();
        self.load_profile_heads();
        self.offline_rename = None;
        self.servers.text(Field::OfflineRename).set(String::new());
        self.servers.set_field_invalid(Field::OfflineRename, false);
    }

    fn offline_key(&self, name: &str) -> String {
        profile_heads::offline_key(name)
    }

    /// The field and its two buttons, for the classic look's rows.
    fn rename_actions(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(self.field(
                Field::OfflineRename,
                tr("A name, 3 to 16 letters"),
                180.,
                cx,
            ))
            .child(
                button("offline-rename-save", "Save", Kind::Secondary, &t).on_click(
                    cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.commit_offline_rename();
                        cx.notify();
                    }),
                ),
            )
            .child(
                button("offline-rename-cancel", "Cancel", Kind::Ghost, &t).on_click(
                    cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.cancel_offline_rename(cx);
                    }),
                ),
            )
            .into_any_element()
    }

    /// Settings' rows for the offline names, after the Microsoft accounts: each with Play as
    /// and Forget, then a field to add one.
    pub(super) fn offline_rows(&mut self, mut n: usize, cx: &mut Context<Self>) -> (Div, usize) {
        let t = self.theme;
        let mut rows = div().flex().flex_col();
        let now = self.accounts.player();
        for (i, name) in self.accounts.offline.clone().into_iter().enumerate() {
            let plays = now == Player::Offline(name.clone());
            n += 1;
            if self.offline_rename.as_deref() == Some(name.as_str()) {
                rows = rows.child(index_row(
                    n,
                    name,
                    Some(div().child(tr("Rename")).into_any_element()),
                    Some(self.rename_actions(cx)),
                    &t,
                ));
                continue;
            }
            let mut actions = div().flex().gap(px(4.));
            if !plays {
                let who = Player::Offline(name.clone());
                actions = actions.child(
                    button(format!("offline-play-{i}"), "Play as", Kind::Secondary, &t).on_click(
                        cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.play_as(who.clone());
                            cx.notify();
                        }),
                    ),
                );
            }
            let dressed = name.clone();
            actions = actions.child(
                button(format!("offline-skin-{i}"), "Skin…", Kind::Ghost, &t).on_click(
                    cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.open_skin(dressed.clone());
                        cx.notify();
                    }),
                ),
            );
            let edited = name.clone();
            actions = actions.child(
                button(format!("offline-rename-{i}"), "Rename", Kind::Ghost, &t).on_click(
                    cx.listener(move |this, _: &ClickEvent, window, cx| {
                        this.begin_offline_rename(&edited, window, cx);
                        cx.notify();
                    }),
                ),
            );
            let forget = name.clone();
            actions = actions.child(
                button(format!("offline-forget-{i}"), "Forget", Kind::Ghost, &t).on_click(                        cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.ask_account_delete(AccountDelete::Offline(forget.clone()));
                        cx.notify();
                    }),
                ),
            );
            rows = rows.child(index_row(
                n,
                name,
                Some(
                    div()
                        .child(tr(if plays {
                            "Offline name · plays"
                        } else {
                            "Offline name"
                        }))
                        .into_any_element(),
                ),
                Some(actions.into_any_element()),
                &t,
            ));
        }
        rows = rows.children(self.skin_panel(cx));
        // The name the game made up can't be forgotten — it's what an offline player is —
        // only rolled for a new one.
        if let Some(made_up) = self.offline_name.clone() {
            let plays = now == Player::GameName;
            n += 1;
            let renaming = self.offline_rename.as_deref() == Some(made_up.as_str());
            let right: AnyElement = if renaming {
                self.rename_actions(cx)
            } else {
                let edited = made_up.clone();
                div()
                    .flex()
                    .gap(px(4.))
                    .child(
                        button("game-name-rename", "Rename", Kind::Ghost, &t).on_click(
                            cx.listener(move |this, _: &ClickEvent, window, cx| {
                                this.begin_offline_rename(&edited, window, cx);
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        button("game-name-new", "New name", Kind::Ghost, &t).on_click(
                            cx.listener(|this, _: &ClickEvent, _, cx| {
                                match instances::new_player_name() {
                                    Ok(name) => this.offline_name = Some(name),
                                    Err(e) => this.sign_in_state = Some(SignInState::Done(Err(e))),
                                }
                                cx.notify();
                            }),
                        ),
                    )
                    .into_any_element()
            };
            rows = rows.child(index_row(
                n,
                made_up,
                Some(
                    div()
                        .child(tr(if renaming {
                            "Rename"
                        } else if plays {
                            "Offline name · the game's own · plays"
                        } else {
                            "Offline name · the game's own"
                        }))
                        .into_any_element(),
                ),
                Some(right),
                &t,
            ));
        }
        n += 1;
        let add = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(self.field(Field::OfflineName, tr("A name, 3 to 16 letters"), 220., cx))
            .child(
                button("offline-add", "Add", Kind::Secondary, &t).on_click(cx.listener(
                    |this, _: &ClickEvent, _, cx| {
                        this.add_offline_name();
                        cx.notify();
                    },
                )),
            );
        rows = rows.child(index_row(
            n,
            tr("Offline name"),
            Some(
                div()
                    .child(tr(
                        "Play offline under another name, and switch from the top bar. Online \
                         servers don't take offline names.",
                    ))
                    .into_any_element(),
            ),
            Some(add.into_any_element()),
            &t,
        ));
        (rows, n)
    }
}
