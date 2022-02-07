use gdk::ModifierType;
use gdk::keys::Key;

use quickmd::ui::action::{Action, Keymaps};

#[test]
fn test_default_keybindings() {
    let keymaps = Keymaps::default();

    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("j")), Action::SmallScrollDown);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("k")), Action::SmallScrollUp);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("J")), Action::BigScrollDown);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("K")), Action::BigScrollUp);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("g")), Action::ScrollToTop);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("G")), Action::ScrollToBottom);
}

#[test]
fn test_shift_and_capital_letter_equivalence() {
    let keymaps = Keymaps::default();

    let big_j = keymaps.get_action(ModifierType::empty(), Key::from_name("J"));
    assert_eq!(big_j, Action::BigScrollDown);

    let shift_j = keymaps.get_action(ModifierType::SHIFT_MASK, Key::from_name("j"));
    assert_eq!(shift_j, Action::BigScrollDown);
}
