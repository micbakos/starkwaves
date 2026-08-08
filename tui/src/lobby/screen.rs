use crate::app::services::Services;
use crate::app::types::CoreState;
use crate::app::types::Intent::{OnAccountLoggedOut, OnGameUpdate, OnNav, OnShowToast};
use crate::lobby::types::{
    AccountMenu, Effect, ExitLobbyPopup, Intent, LobbyState, LobbyVariant, State,
};
use crate::popup::render_popup;
use crate::types::error::TuiError;
use crate::types::menu_iterable::MenuIterable;
use crate::types::result::Result;
use crate::types::screen::Screen;
use crate::types::{AppEffect, AppIntent};
use crate::utils::{format_address_felt, window_ratio};
use clipboard_rs::{Clipboard, ClipboardContext};
use crossterm::event::{KeyCode, KeyEvent};
use log::debug;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::prelude::{Line, Style};
use ratatui::widgets::{Block, Paragraph, Wrap};
use starkwaves_client::game::game::{Game, GameUpdate};
use starkwaves_client::types::board_size::BoardSize;
use starkwaves_client::types::game_state::GameState;
use starkwaves_client::types::lobby::Lobbies;
use std::sync::Arc;
use std::time::Duration;
use strum::VariantArray;
use tokio::sync::Mutex;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use super::types::ExitLobbyPopupAction;

pub struct LobbyScreen;

impl Screen for LobbyScreen {
    type Intent = Intent;
    type Effect = Effect;
    type State = State;

    fn reduce(
        state: &Self::State,
        intent: Self::Intent,
        core: &CoreState,
    ) -> (Self::State, Vec<AppEffect>) {
        let mut new_state = state.clone();
        let mut effects = vec![];
        match intent {
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
                if let Some(popup) = &state.exit_lobby_popup {
                    match popup.selected_action {
                        ExitLobbyPopupAction::Exit => {
                            effects.push(Effect::RequestExitLobby(popup.lobby_size).into());
                            new_state.exit_lobby_popup = None;
                        }
                        ExitLobbyPopupAction::Cancel => {
                            new_state.exit_lobby_popup = None;
                        }
                    }
                } else if let Some(item) = new_state.selected_account_menu_item {
                    match item {
                        AccountMenu::Copy => {
                            let address_text = state.account.address.to_fixed_hex_string();
                            effects.push(Effect::RequestCopyToClipboard(address_text).into());
                        }
                        AccountMenu::Logout => {
                            effects.push(Effect::RequestLogout.into());
                        }
                    }
                } else if let Some(item) = new_state.selected_lobby
                    && let Some(received) = state.lobby.as_received()
                {
                    let player_address = core
                        .account
                        .as_logged_in()
                        .map(|a| a.address)
                        .expect("Expected to be logged in");

                    if let Some(lobby_size) = received.player_lobby(player_address) {
                        new_state.exit_lobby_popup = Some(ExitLobbyPopup::new(lobby_size));
                    } else {
                        effects.push(Effect::RequestJoinLobby(item).into());
                    }
                }
            }
            Intent::OnTimeToRefreshLobbyState => {
                effects.push(Effect::StartPollingLobbies.into());
            }
            Intent::OnJoinedLobby(board_size) => {
                if let Some(lobbies_state) = new_state.lobby.as_received_mut() {
                    lobbies_state.join(board_size, state.account.address)
                }
            }
            Intent::OnSelectNextExitLobbyPopupMenuItem => {
                if let Some(popup) = new_state.exit_lobby_popup.as_mut() {
                    popup.selected_action = popup.selected_action.next();
                }
            }
            Intent::OnSelectPrevExitLobbyPopupMenuItem => {
                if let Some(popup) = new_state.exit_lobby_popup.as_mut() {
                    popup.selected_action = popup.selected_action.prev();
                }
            }
            Intent::OnExitedLobby(board_size) => {
                if let Some(lobbies_state) = new_state.lobby.as_received_mut() {
                    lobbies_state.exit(board_size);
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
            KeyCode::Right => {
                if state.exit_lobby_popup.is_some() {
                    Some(Intent::OnSelectNextExitLobbyPopupMenuItem)
                } else {
                    Some(Intent::OnMoveFocusToAccount)
                }
            }
            KeyCode::Left => {
                if state.exit_lobby_popup.is_some() {
                    Some(Intent::OnSelectPrevExitLobbyPopupMenuItem)
                } else {
                    Some(Intent::OnMoveFocusToLobby)
                }
            }
            KeyCode::Enter => Some(Intent::OnSelectionClicked),
            _ => None,
        }
    }

    fn on_push_effect() -> Option<Self::Effect> {
        Some(Effect::StartPollingLobbies)
    }

    fn on_pop_effect() -> Option<Self::Effect> {
        Some(Effect::StopPollingLobbies)
    }

    async fn run(
        effect: Self::Effect,
        services: Arc<Services>,
        intents: UnboundedSender<AppIntent>,
    ) -> std::result::Result<(), SendError<AppIntent>> {
        match effect {
            Effect::StartPollingLobbies => {
                start_lobbies_polling(services, intents);
            }
            Effect::StopPollingLobbies => {
                stop_lobbies_polling(services);
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
                    intents.send(OnAccountLoggedOut.into())?;
                    intents.send(
                        OnNav(crate::types::nav::NavCommand::ResetTo(splash.into())).into(),
                    )?;
                }
            }
            Effect::RequestJoinLobby(lobby_variant) => {
                stop_lobbies_polling(services.clone());

                let board_size = BoardSize::from(lobby_variant);

                let player = {
                    let guard = services.player.read().unwrap();

                    guard
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| todo!("No player found, redirect to login"))
                };

                let on_chain = services.on_chain.clone();

                let (updates_sender, mut updates_receiver) =
                    mpsc::unbounded_channel::<GameUpdate>();

                let intents_sender = intents.clone();
                tokio::spawn(async move {
                    while let Some(update) = updates_receiver.recv().await {
                        intents_sender
                            .send(OnGameUpdate(update).into())
                            .expect("Expect game update to reach app");
                    }
                });

                let game_handle_result: Result<Arc<Mutex<Game>>> = Game::join(
                    on_chain.contract_address,
                    on_chain.ws_url,
                    player,
                    board_size,
                    updates_sender,
                )
                .await
                .map_err(|e| e.into());

                match game_handle_result {
                    Ok(game_handle) => {
                        {
                            let mut game_guard = services.in_game.write().unwrap();
                            *game_guard = Some(game_handle.clone());
                        }

                        let game = game_handle.lock().await;
                        match game.state() {
                            GameState::InLobby(board_size) => {
                                intents.send(Intent::OnJoinedLobby(board_size).into())?;
                                start_lobbies_polling(services, intents);
                            }
                            GameState::InGame(game_data) => {}
                        }
                    }
                    Err(err) => {
                        intents
                            .send(OnShowToast(format!("Unable to join game. {}", err)).into())?;
                        start_lobbies_polling(services, intents);
                    }
                }
            }
            Effect::RequestExitLobby(board_size) => {
                stop_lobbies_polling(services.clone());
                let game_handle = services
                    .in_game
                    .read()
                    .unwrap()
                    .clone()
                    .expect("Game instance should exist");
                let game = game_handle.lock().await;

                if let Err(error) = game
                    .exit_lobby(&board_size)
                    .await
                    .map_err(Into::<TuiError>::into)
                {
                    intents.send(OnShowToast(format!("Unable to exit lobby. {}", error)).into())?;
                } else {
                    {
                        let mut game_guard = services.in_game.write().unwrap();
                        *game_guard = None;
                    }
                    intents.send(Intent::OnExitedLobby(board_size).into())?;
                }
                start_lobbies_polling(services, intents);
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
                LobbyVariant::Six => "(6 x 6) • Small",
                LobbyVariant::Eight => "(8 x 8) • Small",
                LobbyVariant::Ten => "(10x10) • Normal",
                LobbyVariant::Twelve => "(12x12) • Large",
                LobbyVariant::Fourteen => "(14x14) • Large",
                LobbyVariant::Twenty => "(20x20) • Large",
            }
            .to_string();

            let player_label = match lobby_state {
                LobbyState::Idle => "".to_string(),
                LobbyState::Resolving => "Resolving players...".to_string(),
                LobbyState::Received(lobbies) => {
                    if let Some(player) = lobbies.lobby((*variant).into()) {
                        if player == state.account.address {
                            "Awaiting opponent...".to_string()
                        } else {
                            player.to_fixed_hex_string()
                        }
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

    if let Some(popup) = &state.exit_lobby_popup {
        render_popup(
            frame,
            lobbies_area,
            Some("Exit Lobby?"),
            format!("You are about to exit lobby {}", popup.lobby_size),
            &popup.selected_action,
            ExitLobbyPopupAction::VARIANTS,
        );
    }
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

fn start_lobbies_polling(services: Arc<Services>, intents: UnboundedSender<AppIntent>) {
    let mut handle_guard = services.lobby_polling.write().unwrap();
    if handle_guard.is_some() {
        debug!("Lobby polling already present");
        return;
    }

    debug!("Start polling lobbies");
    let handle = poll_lobbies(services.clone(), intents);
    *handle_guard = Some(handle);
}

fn stop_lobbies_polling(services: Arc<Services>) {
    if let Some(lobby_polling_guard) = services.lobby_polling.write().unwrap().take() {
        debug!("Stop polling lobbies");
        lobby_polling_guard.abort();
    }
}

fn poll_lobbies(services: Arc<Services>, intents: UnboundedSender<AppIntent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            intents.send(Intent::OnUpdateLobbyState(LobbyState::Resolving).into());

            let player = {
                let player_guard = services.player.read().unwrap();

                player_guard
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| todo!("No player found, redirect to login"))
            };

            let player_address = player.address();
            let result: Result<Lobbies> =
                Game::get_lobbies(services.on_chain.contract_address, &player)
                    .await
                    .map_err(|e| e.into());

            match result {
                Ok(lobbies) => {
                    let no_in_game = services.in_game.read().unwrap().is_none();

                    intents.send(
                        Intent::OnUpdateLobbyState(LobbyState::Received(lobbies.clone())).into(),
                    );

                    // In case we receive an update that the player is in lobby from probably
                    // previous session (game handle doesn't exist yet),
                    // start the request to await opponent and store game handle
                    if let Some(board_size) = lobbies.player_lobby(player_address)
                        && no_in_game
                    {
                        let on_chain = services.on_chain.clone();
                        let (updates_sender, mut updates_receiver) =
                            mpsc::unbounded_channel::<GameUpdate>();

                        let intents_sender = intents.clone();
                        tokio::spawn(async move {
                            while let Some(update) = updates_receiver.recv().await {
                                intents_sender
                                    .send(OnGameUpdate(update).into())
                                    .expect("Expect game update to reach app");
                            }
                        });

                        let game_handle = Game::await_opponent(
                            on_chain.contract_address,
                            on_chain.ws_url,
                            player,
                            board_size,
                            updates_sender,
                            None,
                        )
                        .await;

                        {
                            let mut game_guard = services.in_game.write().unwrap();
                            *game_guard = Some(game_handle.clone());
                        }
                    }
                }
                Err(e) => {
                    intents.send(Intent::OnUpdateLobbyState(LobbyState::Idle).into());
                    intents.send(OnShowToast(e.to_string()).into());
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    })
}
