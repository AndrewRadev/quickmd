use gdk::ModifierType;
use gdk::keys::Key;

use quickmd::ui::action::{Action, Keymaps};

#[test]
fn test_default_keybindings() {
    let keymaps = Keymaps::default();

    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('j')), Action::SmallScrollDown);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('k')), Action::SmallScrollUp);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('J')), Action::BigScrollDown);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('K')), Action::BigScrollUp);

    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('e')), Action::LaunchEditor);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('E')), Action::ExecEditor);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('+')), Action::ZoomIn);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('-')), Action::ZoomOut);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('=')), Action::ZoomReset);

    assert_eq!(keymaps.get_action(ModifierType::CONTROL_MASK, Key::from_unicode('q')), Action::Quit);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("F1")), Action::ShowHelp);

}

#[test]
fn test_shift_and_capital_letter_equivalence_when_getting() {
    let keymaps = Keymaps::default();

    let big_j = keymaps.get_action(ModifierType::empty(), Key::from_name("J"));
    assert_eq!(big_j, Action::BigScrollDown);

    let shift_j = keymaps.get_action(ModifierType::SHIFT_MASK, Key::from_name("j"));
    assert_eq!(shift_j, Action::BigScrollDown);
}

#[test]
fn test_setting_custom_mappings() {
    let mut keymaps = Keymaps::default();

    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("q")), Action::Noop);

    keymaps.set_action(ModifierType::empty(), Key::from_name("q"), Action::Quit);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("q")), Action::Quit);

    keymaps.set_action(ModifierType::empty(), Key::from_name("q"), Action::ZoomIn);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("q")), Action::ZoomIn);
}

#[test]
fn test_unsetting_a_default_mapping() {
    let mut keymaps = Keymaps::default();

    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("j")), Action::SmallScrollDown);

    keymaps.set_action(ModifierType::empty(), Key::from_name("j"), Action::Noop);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_name("j")), Action::Noop);
}
