//! Main Menu System for RTGC-0.8
//! Handles main menu, new game, continue, options, exit

use crate::game::character_creation::CharacterCreationManager;
use crate::game::save::{SaveSystem, SaveMetadata};
use std::path::PathBuf;

/// Main menu states
#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    /// Main menu (New Game, Continue, Options, Exit)
    MainMenu,
    /// Character creation in progress
    CharacterCreation,
    /// Loading screen
    Loading,
    /// In-game menu (paused)
    Paused,
}

/// Main menu manager
pub struct MainMenu {
    state: MenuState,
    hovered_button: Option<MenuButton>,
    character_creation: Option<CharacterCreationManager>,
    saves: Vec<SaveMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuButton {
    NewGame,
    Continue,
    Options,
    Exit,
    Resume,
    SaveGame,
    LoadGame,
    Settings,
    Back,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut menu = Self {
            state: MenuState::MainMenu,
            hovered_button: None,
            character_creation: None,
            saves: Vec::new(),
        };
        menu.load_saves();
        menu
    }

    /// Load save metadata for "Continue" button
    fn load_saves(&mut self) {
        // Will be implemented with actual save system integration
        self.saves.clear();
        // Placeholder: check if saves exist
        let save_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("saves");
        
        if save_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&save_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().map_or(false, |ext| ext == "json") {
                        // Parse metadata from save file
                        // For now, just count them
                        self.saves.push(SaveMetadata {
                            slot: self.saves.len() as u32,
                            player_name: "Player".to_string(),
                            location: "Unknown".to_string(),
                            playtime_seconds: 0,
                            real_time_saved: chrono::Local::now(),
                            money: 0,
                        });
                    }
                }
            }
        }
    }

    /// Get current menu state
    pub fn state(&self) -> &MenuState {
        &self.state
    }

    /// Start new game - initialize character creation
    pub fn start_new_game(&mut self) {
        self.state = MenuState::CharacterCreation;
        self.character_creation = Some(CharacterCreationManager::new());
    }

    /// Get mutable reference to character creation manager
    pub fn character_creation_mut(&mut self) -> Option<&mut CharacterCreationManager> {
        self.character_creation.as_mut()
    }

    /// Check if character creation is complete
    pub fn is_character_creation_complete(&self) -> bool {
        self.character_creation
            .as_ref()
            .map_or(false, |cc| cc.is_complete())
    }

    /// Get character creation data if complete
    pub fn get_character_data(&self) -> Option<&crate::game::character_creation::CharacterCreationData> {
        self.character_creation
            .as_ref()
            .and_then(|cc| cc.get_final_data())
    }

    /// Continue game - load most recent save
    pub fn continue_game(&mut self) -> Option<PathBuf> {
        if self.saves.is_empty() {
            return None;
        }
        
        // Load most recent save
        let save_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("saves");
        
        // Find most recent save file
        let mut latest_save: Option<(PathBuf, std::time::SystemTime)> = None;
        
        if save_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&save_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "json") {
                        if let Ok(meta) = entry.metadata() {
                            if let Ok(modified) = meta.modified() {
                                match &latest_save {
                                    None => latest_save = Some((path, modified)),
                                    Some((_, latest_time)) => {
                                        if modified > *latest_time {
                                            latest_save = Some((path, modified));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        latest_save.map(|(path, _)| path)
    }

    /// Handle button hover
    pub fn hover_button(&mut self, button: MenuButton) {
        self.hovered_button = Some(button);
    }

    /// Handle button click
    pub fn click_button(&mut self, button: MenuButton) -> MenuAction {
        match button {
            MenuButton::NewGame => {
                self.start_new_game();
                MenuAction::None
            }
            MenuButton::Continue => {
                if let Some(path) = self.continue_game() {
                    MenuAction::LoadGame(path)
                } else {
                    MenuAction::None
                }
            }
            MenuButton::Exit => MenuAction::Exit,
            MenuButton::Resume => MenuAction::Resume,
            MenuButton::SaveGame => MenuAction::SaveGame,
            MenuButton::LoadGame => MenuAction::OpenLoadMenu,
            MenuButton::Options | MenuButton::Settings => MenuAction::OpenSettings,
            MenuButton::Back => {
                self.state = MenuState::MainMenu;
                MenuAction::None
            }
        }
    }

    /// Update menu (handle character creation progress)
    pub fn update(&mut self, dt: f32) {
        if let Some(cc) = &mut self.character_creation {
            cc.update(dt);
            
            // If character creation is complete, transition to loading
            if cc.is_complete() {
                self.state = MenuState::Loading;
            }
        }
    }

    /// Render menu UI (placeholder - actual rendering in renderer)
    pub fn render(&self) -> MenuRenderData {
        match self.state {
            MenuState::MainMenu => MenuRenderData::MainMenu {
                hovered: self.hovered_button,
                has_saves: !self.saves.is_empty(),
            },
            MenuState::CharacterCreation => MenuRenderData::CharacterCreation,
            MenuState::Loading => MenuRenderData::Loading,
            MenuState::Paused => MenuRenderData::Paused {
                hovered: self.hovered_button,
            },
        }
    }
}

/// Actions to perform based on menu interaction
#[derive(Debug)]
pub enum MenuAction {
    None,
    Exit,
    Resume,
    LoadGame(PathBuf),
    SaveGame,
    OpenLoadMenu,
    OpenSettings,
    StartGame,
}

/// Data for rendering the menu
#[derive(Debug)]
pub enum MenuRenderData {
    MainMenu {
        hovered: Option<MenuButton>,
        has_saves: bool,
    },
    CharacterCreation,
    Loading,
    Paused {
        hovered: Option<MenuButton>,
    },
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_menu_creation() {
        let menu = MainMenu::new();
        assert_eq!(menu.state(), &MenuState::MainMenu);
    }

    #[test]
    fn test_start_new_game() {
        let mut menu = MainMenu::new();
        menu.start_new_game();
        assert_eq!(menu.state(), &MenuState::CharacterCreation);
        assert!(menu.character_creation.is_some());
    }
}
