//! Actions on the UI triggered by keybindings

use std::collections::HashMap;

use gdk::ModifierType;
use gdk::keys::{self, Key};

/// Mappable actions
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Placeholder action to allow unmapping keys
    Noop,

    /// Scroll up by a small step
    SmallScrollUp,
    /// Scroll down by a small step
    SmallScrollDown,

    /// Scroll up by a large step
    BigScrollUp,
    /// Scroll down by a large step
    BigScrollDown,

    /// Scroll to the top of the document
    ScrollToTop,
    /// Scroll to the bottom of the document
    ScrollToBottom,

    /// Quit the entire application
    Quit,

    /// Launch an editor instance if it's configured
    LaunchEditor,
    /// Exec the current process into an editor instance if it's configured (and it's possible on
    /// the OS)
    ExecEditor,

    /// Zoom the browser in by 10%
    ZoomIn,
    /// Zoom the browser out by 10%
    ZoomOut,
    /// Reset the zoom level to the configured starting point
    ZoomReset,

    /// Show a help popup
    ShowHelp,
}

/// A mapping from key bindings to all the different UI actions. Initialized with a full set of
/// defaults, which can be overridden by configuration.
///
pub struct Keymaps {
    mappings: HashMap<(ModifierType, Key), Action>,
}

impl Default for Keymaps {
    fn default() -> Self {
        let mut mappings = HashMap::new();

        // Scroll with j/k, J/K:
        mappings.insert((ModifierType::empty(),    keys::constants::j), Action::SmallScrollDown);
        mappings.insert((ModifierType::empty(),    keys::constants::k), Action::SmallScrollUp);
        mappings.insert((ModifierType::SHIFT_MASK, keys::constants::j), Action::BigScrollDown);
        mappings.insert((ModifierType::SHIFT_MASK, keys::constants::k), Action::BigScrollUp);
        // Jump to the top/bottom with g/G
        mappings.insert((ModifierType::empty(),    keys::constants::g), Action::ScrollToTop);
        mappings.insert((ModifierType::SHIFT_MASK, keys::constants::g), Action::ScrollToBottom);
        // Ctrl+Q to quit
        mappings.insert((ModifierType::CONTROL_MASK, keys::constants::q), Action::Quit);
        // e, E for editor integration
        mappings.insert((ModifierType::empty(),    keys::constants::e), Action::LaunchEditor);
        mappings.insert((ModifierType::SHIFT_MASK, keys::constants::e), Action::ExecEditor);
        // +/-/= for zoom control
        mappings.insert((ModifierType::empty(), keys::constants::plus), Action::ZoomIn);
        mappings.insert((ModifierType::empty(), keys::constants::minus), Action::ZoomOut);
        mappings.insert((ModifierType::empty(), keys::constants::equal), Action::ZoomReset);
        // F1 to show help popup
        mappings.insert((ModifierType::empty(), keys::constants::F1), Action::ShowHelp);

        Self { mappings }
    }
}

impl Keymaps {
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
        if let Some(letter) = key.to_unicode() {
            if letter.is_alphabetic() && letter.is_uppercase() {
                key = key.to_lower();
                modifiers.insert(ModifierType::SHIFT_MASK);
            }
        }

        (key, modifiers)
    }
}
