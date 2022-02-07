//! Actions on the UI triggered by keybindings or the mouse.

use gdk::ModifierType;
use gdk::keys::Key;

/// Mappable actions
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Noop,
    SmallScrollUp,
    SmallScrollDown,
    BigScrollUp,
    BigScrollDown,
    ScrollToTop,
    ScrollToBottom,
}

pub struct Keymaps {
    mappings: Vec<(ModifierType, Key, Action)>,
}

impl Default for Keymaps {
    fn default() -> Self {
        let mut mappings = Vec::new();

        // Scroll with j/k, J/K:
        mappings.push((ModifierType::empty(),    Key::from_name("j"), Action::SmallScrollDown));
        mappings.push((ModifierType::empty(),    Key::from_name("k"), Action::SmallScrollUp));
        mappings.push((ModifierType::SHIFT_MASK, Key::from_name("j"), Action::BigScrollDown));
        mappings.push((ModifierType::SHIFT_MASK, Key::from_name("k"), Action::BigScrollUp));
        // Jump to the top/bottom with g/G
        mappings.push((ModifierType::empty(),    Key::from_name("g"), Action::ScrollToTop));
        mappings.push((ModifierType::SHIFT_MASK, Key::from_name("g"), Action::ScrollToBottom));

        Self { mappings }
    }
}

impl Keymaps {
    pub fn get_action(&self, mut modifiers: ModifierType, mut key: Key) -> Action {
        if key.is_upper() {
            key = key.to_lower();
            modifiers.insert(ModifierType::SHIFT_MASK);
        }

        self.mappings.iter().find_map(|(mapping_modifiers, mapping_key, action)| {
            if modifiers == *mapping_modifiers && mapping_key == &key {
                Some(action)
            } else {
                None
            }
        }).cloned().unwrap_or(Action::Noop)
    }
}
