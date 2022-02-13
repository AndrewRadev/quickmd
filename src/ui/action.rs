//! Actions on the UI triggered by keybindings.

use std::collections::HashMap;

use anyhow::anyhow;
use gdk::ModifierType;
use gdk::keys::{self, Key};
use serde::{Serialize, Deserialize};

use crate::input::MappingDefinition;

/// Mappable actions
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Placeholder action to allow unmapping keys
    Noop,

    /// Scroll up by a small step. Default: `k`
    SmallScrollUp,
    /// Scroll down by a small step. Default: `j`
    SmallScrollDown,

    /// Scroll up by a large step. Default: `K`
    BigScrollUp,
    /// Scroll down by a large step. Default: `J`
    BigScrollDown,

    /// Scroll to the top of the document. Default: `g`
    ScrollToTop,
    /// Scroll to the bottom of the document. Default: `G`
    ScrollToBottom,

    /// Quit the entire application. Default: `ctrl+q`
    Quit,

    /// Launch an editor instance if it's configured. Default: `e`
    LaunchEditor,
    /// Exec the current process into an editor instance if it's configured (and it's possible on
    /// the OS). Default: `E`
    ExecEditor,

    /// Zoom the browser in by 10%. Default: `+`
    ZoomIn,
    /// Zoom the browser out by 10%. Default: `-`
    ZoomOut,
    /// Reset the zoom level to the configured starting point. Default: `=`
    ZoomReset,

    /// Show a help popup. Default: `F1`
    ShowHelp,
}

/// A mapping from key bindings to all the different UI actions. Initialized with a full set of
/// defaults, which can be overridden by configuration.
///
#[derive(Clone)]
pub struct Keymaps {
    mappings: HashMap<(ModifierType, Key), Action>,
}

impl Default for Keymaps {
    fn default() -> Self {
        let mut keymaps = Self::new();

        // Scroll with j/k, J/K:
        keymaps.set_action(ModifierType::empty(),    keys::constants::j, Action::SmallScrollDown);
        keymaps.set_action(ModifierType::empty(),    keys::constants::k, Action::SmallScrollUp);
        keymaps.set_action(ModifierType::SHIFT_MASK, keys::constants::j, Action::BigScrollDown);
        keymaps.set_action(ModifierType::SHIFT_MASK, keys::constants::k, Action::BigScrollUp);
        // Jump to the top/bottom with g/G
        keymaps.set_action(ModifierType::empty(),    keys::constants::g, Action::ScrollToTop);
        keymaps.set_action(ModifierType::SHIFT_MASK, keys::constants::g, Action::ScrollToBottom);
        // Ctrl+Q to quit
        keymaps.set_action(ModifierType::CONTROL_MASK, keys::constants::q, Action::Quit);
        // e, E for editor integration
        keymaps.set_action(ModifierType::empty(),    keys::constants::e, Action::LaunchEditor);
        keymaps.set_action(ModifierType::SHIFT_MASK, keys::constants::e, Action::ExecEditor);
        // +/-/= for zoom control
        keymaps.set_action(ModifierType::empty(), keys::constants::plus, Action::ZoomIn);
        keymaps.set_action(ModifierType::empty(), keys::constants::minus, Action::ZoomOut);
        keymaps.set_action(ModifierType::empty(), keys::constants::equal, Action::ZoomReset);
        // F1 to show help popup
        keymaps.set_action(ModifierType::empty(), keys::constants::F1, Action::ShowHelp);

        keymaps
    }
}

impl Keymaps {
    fn new() -> Self {
        Self { mappings: HashMap::new() }
    }

    /// Parse the given mappings as described in [`crate::input::Config`]
    ///
    pub fn add_config_mappings(&mut self, mappings: &Vec<MappingDefinition>) -> anyhow::Result<()> {
        for mapping in mappings {
            let mut modifiers = ModifierType::empty();
            for m in &mapping.modifiers {
                match m.as_str() {
                    "control" => { modifiers |= ModifierType::CONTROL_MASK; }
                    "shift"   => { modifiers |= ModifierType::SHIFT_MASK; }
                    "alt"     => { modifiers |= ModifierType::MOD1_MASK; }
                    _ => {
                        { return Err(anyhow!("Unknown modifier: {}", m)); }
                    },
                }
            }

            let key =
                if let Some(c) = mapping.key_char {
                    Key::from_unicode(c)
                } else if let Some(name) = &mapping.key_name {
                    Key::from_name(name)
                } else {
                    return Err(anyhow!("No `key_char` or `key_name` given: {:?}", mapping));
                };

            self.set_action(modifiers, key, mapping.action.clone());
        }

        Ok(())
    }

    /// Get the action corresponding to the given modifiers and key. Uppercase unicode letters like
    /// are normalized to a lowercase letter + shift.
    ///
    pub fn get_action(&self, modifiers: ModifierType, key: Key) -> Action {
        let (key, modifiers) = Self::normalize_input(key, modifiers);
        self.mappings.get(&(modifiers, key)).cloned().unwrap_or(Action::Noop)
    }

    /// Set the action corresponding to the given modifiers and key. Could override existing
    /// actions. Setting `Action::Noop` is the way to "unmap" a keybinding. Uppercase unicode
    /// letters like are normalized to a lowercase letter + shift.
    ///
    pub fn set_action(&mut self, modifiers: ModifierType, key: Key, action: Action) {
        let (key, modifiers) = Self::normalize_input(key, modifiers);
        self.mappings.insert((modifiers, key), action);
    }

    fn normalize_input(mut key: Key, mut modifiers: ModifierType) -> (Key, ModifierType) {
        // If we get something considered an "upper" key, that means shift is being held. This is
        // not just for A -> S-a, but also for + -> = (though the + is not transformed).
        if key.is_upper() {
            key = key.to_lower();
            modifiers.insert(ModifierType::SHIFT_MASK);
        }

        (key, modifiers)
    }
}
