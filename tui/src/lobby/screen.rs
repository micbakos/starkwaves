use crate::app::services::Services;
use crate::app::types::CoreState;
use crate::app::types::Intent::{OnNav, OnShowToast};
use crate::lobby::types::{AccountMenu, Effect, Intent, LobbyState, LobbyVariant, State};
use crate::types::menu_iterable::MenuIterable;
use crate::types::result::Result;
use crate::types::screen::Screen;
use crate::types::{AppEffect, AppIntent};
use crate::utils::{format_address_felt, window_ratio};
use clipboard_rs::{Clipboard, ClipboardContext};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::prelude::{Line, Style};
use ratatui::widgets::{Block, Paragraph, Wrap};
use starkwaves_client::game::game::Game;
use starkwaves_client::types::board_size::BoardSize;
use starkwaves_client::types::lobby::Lobbies;
use std::sync::Arc;
use strum::VariantArray;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::SendError;

pub struct LobbyScreen;

impl Screen for LobbyScreen {
    type Intent = Intent;
    type Effect = Effect;
    type State = State;

    fn reduce(
        state: &Self::State,
        intent: Self::Intent,
        _core: &CoreState,
    ) -> (Self::State, Vec<AppEffect>) {
        let mut new_state = state.clone();
        let mut effects = vec![];
        match intent {
            Intent::OnStart => effects.push(Effect::RequestGetLobbies.into()),
            Intent::OnUpdateLobbyState(state) => new_state.lobby = state,
            Intent::OnSelectPreviousLobby => {
                if let Some(lobby_variant) = new_state.selected_lobby {
                    new_state.selected_lobby = Some(lobby_variant.prev());
                } else {
                    new_state.selected_lobby = LobbyVariant::VARIANTS.last().cloned();
                }
            }
            Intent::OnSelectNextLobby => {
                if let Some(lobby_variant) = new_state.selected_lobby {
                    new_state.selected_lobby = Some(lobby_variant.next());
                } else {
                    new_state.selected_lobby = LobbyVariant::VARIANTS.first().cloned();
                }
            }
            Intent::OnMoveFocusToAccount => {
                new_state.selected_lobby = None;
                new_state.selected_account_menu_item = Some(AccountMenu::Copy);
            }
            Intent::OnMoveFocusToLobby => {
                new_state.selected_lobby = Some(LobbyVariant::first());
                new_state.selected_account_menu_item = None;
            }
            Intent::OnSelectNextAccountMenuItem => {
                if let Some(item) = new_state.selected_account_menu_item {
                    new_state.selected_account_menu_item = Some(item.next());
                } else {
                    new_state.selected_account_menu_item = AccountMenu::VARIANTS.first().cloned();
                }
            }
            Intent::OnSelectPrevAccountMenuItem => {
                if let Some(item) = new_state.selected_account_menu_item {
                    new_state.selected_account_menu_item = Some(item.prev());
                } else {
                    new_state.selected_account_menu_item = AccountMenu::VARIANTS.last().cloned();
                }
            }
            Intent::OnSelectionClicked => {
                if let Some(item) = new_state.selected_account_menu_item {
                    match item {
                        AccountMenu::Copy => {
                            let address_text = state.account.address.to_fixed_hex_string();
                            effects.push(Effect::RequestCopyToClipboard(address_text).into());
                        }
                        AccountMenu::Logout => {
                            effects.push(Effect::RequestLogout.into());
                        }
                    }
                } else if let Some(item) = new_state.selected_lobby {
                }
            }
        }

        (new_state, effects)
    }

    fn render(state: &Self::State, core: &CoreState, frame: &mut Frame, area: Rect) {
        let [content_area, memo_area] =
            Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)]).areas(area);

        let [lobbies_area, account_area] =
            Layout::horizontal([Constraint::Percentage(66), Constraint::Percentage(34)])
                .areas(content_area);

        let lobbies_block = Block::bordered().title("Lobbies");
        frame.render_widget(lobbies_block, lobbies_area);
        render_lobbies(state, frame, lobbies_area);
        render_account(state, frame, account_area);
        render_memo(state, frame, memo_area);
    }

    fn on_key(key: KeyEvent, state: &Self::State) -> Option<Self::Intent> {
        if !state.lobby.is_received() {
            return None;
        }

        match key.code {
            KeyCode::Up => {
                if state.selected_account_menu_item.is_some() {
                    Some(Intent::OnSelectPrevAccountMenuItem)
                } else {
                    Some(Intent::OnSelectPreviousLobby)
                }
            }
            KeyCode::Down => {
                if state.selected_account_menu_item.is_some() {
                    Some(Intent::OnSelectNextAccountMenuItem)
                } else {
                    Some(Intent::OnSelectNextLobby)
                }
            }
            KeyCode::Right => Some(Intent::OnMoveFocusToAccount),
            KeyCode::Left => Some(Intent::OnMoveFocusToLobby),
            KeyCode::Enter => Some(Intent::OnSelectionClicked),
            _ => None,
        }
    }

    async fn run(
        effect: Self::Effect,
        services: Arc<Services>,
        intents: UnboundedSender<AppIntent>,
    ) -> std::result::Result<(), SendError<AppIntent>> {
        match effect {
            Effect::RequestGetLobbies => {
                intents.send(Intent::OnUpdateLobbyState(LobbyState::Resolving).into())?;

                let player = {
                    let player_guard = services.player.read().unwrap();

                    player_guard
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| todo!("No player found, redirect to login"))
                };

                let result: Result<Lobbies> =
                    Game::get_lobbies(services.on_chain.contract_address, player.as_ref())
                        .await
                        .map_err(|e| e.into());

                match result {
                    Ok(lobbies) => intents
                        .send(Intent::OnUpdateLobbyState(LobbyState::Received(lobbies)).into())?,
                    Err(e) => {
                        intents.send(Intent::OnUpdateLobbyState(LobbyState::Idle).into())?;
                        intents.send(OnShowToast(e.to_string()).into())?;
                    }
                }
            }
            Effect::RequestCopyToClipboard(message) => {
                ClipboardContext::new()
                    .and_then(|c| c.set_text(message))
                    .or_else(|e| {
                        intents.send(
                            OnShowToast(format!("Failed to copy to clipboard: {}", e).to_string())
                                .into(),
                        )
                    })
                    .and_then(|_| {
                        intents.send(OnShowToast("Copied to clipboard.".to_string()).into())
                    })?;
            }
            Effect::RequestLogout => {
                if let Err(e) = services.remove_session().await {
                    intents.send(OnShowToast(e.to_string()).into())?
                } else {
                    let splash = crate::splash::types::State::new();
                    intents.send(OnNav(crate::types::nav::NavCommand::ResetTo(splash.into())).into())?;
                    intents.send(crate::splash::types::Intent::OnStart.into())?;
                }
            }
        }

        Ok(())
    }
}

fn render_account(state: &State, frame: &mut Frame, account_area: Rect) {
    let account_block = Block::bordered().title("Account");
    frame.render_widget(account_block, account_area);

    let avatar_area = account_area.inner(Margin::new(2, 2));
    let avatar_block_size = avatar_area.width * 50 / 100;

    let ratio = window_ratio();
    let avatar_rect = Rect {
        x: avatar_area.x + avatar_block_size / 2,
        y: avatar_area.y + avatar_area.height * 10 / 100,
        width: avatar_block_size,
        height: (avatar_block_size as f32 / ratio) as u16,
    };
    frame.render_widget(Block::bordered(), avatar_rect);

    let username_text = Line::raw(state.account.username.clone()).centered();

    let username_area = Rect {
        x: avatar_area.x,
        y: avatar_rect.bottom(),
        width: avatar_area.width,
        height: 2,
    };
    frame.render_widget(username_text, username_area);

    let selected_style = Style::default().reversed();
    let normal_style = Style::default();

    let copy_style = state
        .selected_account_menu_item
        .map(|m| {
            if m == AccountMenu::Copy {
                selected_style
            } else {
                normal_style
            }
        })
        .unwrap_or(normal_style);

    let logout_style = state
        .selected_account_menu_item
        .map(|m| {
            if m == AccountMenu::Logout {
                selected_style
            } else {
                normal_style
            }
        })
        .unwrap_or(normal_style);

    let address_area = Rect {
        x: avatar_area.x,
        y: username_area.bottom(),
        width: avatar_area.width,
        height: 2,
    };
    let address_text = Line::raw(format_address_felt(state.account.address))
        .centered()
        .style(copy_style);
    frame.render_widget(address_text, address_area);

    let logout_area = Rect {
        x: avatar_area.x,
        y: address_area.bottom(),
        width: avatar_area.width,
        height: 1,
    };
    let logout_text = Line::raw("Logout").centered().style(logout_style);
    frame.render_widget(logout_text, logout_area);
}

fn render_lobbies(state: &State, frame: &mut Frame, lobbies_area: Rect) {
    let per_variant_area =
        Layout::vertical(LobbyVariant::VARIANTS.iter().map(|_| Constraint::Length(1)))
            .split(lobbies_area.inner(Margin::new(2, 1)));

    let selected_style = Style::default().reversed();
    let normal_style = Style::default();

    LobbyVariant::VARIANTS
        .iter()
        .enumerate()
        .for_each(|(index, variant)| {
            let lobby_state = state.lobby.clone();

            let [size, player] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(per_variant_area[index]);

            let size_label = match variant {
                LobbyVariant::Six => "Small • (6x6)",
                LobbyVariant::Eight => "Small • (8x8)",
                LobbyVariant::Ten => "Normal • (10x10)",
                LobbyVariant::Twelve => "Large • (12x12)",
                LobbyVariant::Fourteen => "Large • (14x14)",
                LobbyVariant::Twenty => "Large • (20x20)",
            }
            .to_string();

            let player_label = match lobby_state {
                LobbyState::Idle => "".to_string(),
                LobbyState::Resolving => "Resolving players...".to_string(),
                LobbyState::Received(lobbies) => {
                    if let Some(player) = lobbies.lobby((*variant).into()) {
                        player.0.to_fixed_hex_string()
                    } else {
                        "Empty Lobby".to_string()
                    }
                }
            };

            let style = if let Some(selected) = state.selected_lobby
                && selected == *variant
            {
                selected_style
            } else {
                normal_style
            };

            frame.render_widget(Line::raw(size_label).style(style), size);
            frame.render_widget(Line::raw(player_label).style(style).right_aligned(), player);
        });
}

fn render_memo(state: &State, frame: &mut Frame, memo_area: Rect) {
    let memo = if let Some(variant) = state.selected_lobby {
        let board_size: BoardSize = variant.into();
        let ships = ships_memo(board_size);

        format!(
            "Board of {} cells that include the following ships: {}.\nPress ENTER to join...",
            board_size,
            ships.join(", ")
        )
    } else if let Some(item) = state.selected_account_menu_item {
        match item {
            AccountMenu::Copy => "Press ENTER to copy to clipboard.".to_string(),
            AccountMenu::Logout => "Press ENTER to logout".to_string(),
        }
    } else {
        "".to_string()
    };

    frame.render_widget(Block::bordered().title("Memo"), memo_area);
    frame.render_widget(
        Paragraph::new(memo).wrap(Wrap { trim: true }),
        memo_area.inner(Margin::new(2, 1)),
    );
}

fn ships_memo(board_size: BoardSize) -> Vec<String> {
    let binding = board_size.eligible_ship_kinds();
    let mut kinds: Vec<&starkwaves_client::types::ShipKind> = binding.iter().collect();
    kinds.sort();

    kinds
        .iter()
        .map(|kind| {
            let count = board_size.ship_kinds_count(kind);
            let name = kind.to_string();

            format!("{} x {}", count, name).to_string()
        })
        .collect()
}
