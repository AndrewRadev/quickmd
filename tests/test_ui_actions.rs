use gdk::ModifierType;
use gdk::keys::Key;

use quickmd::input::MappingDefinition;
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

#[test]
fn test_setting_non_alphabetic_shift_keys() {
    let mut keymaps = Keymaps::default();

    keymaps.set_action(ModifierType::empty(), Key::from_unicode('+'), Action::ScrollToTop);
    assert_eq!(keymaps.get_action(ModifierType::empty(), Key::from_unicode('+')), Action::ScrollToTop);
    assert_eq!(keymaps.get_action(ModifierType::SHIFT_MASK, Key::from_unicode('+')), Action::ScrollToTop);

    assert_ne!(keymaps.get_action(ModifierType::SHIFT_MASK, Key::from_unicode('=')), Action::ScrollToTop);
}

#[test]
fn test_successfully_setting_mappings_from_the_config() {
    let mut keymaps = Keymaps::new();

    let mapping = MappingDefinition {
        key_char:  Some('q'),
        key_name:  None,
        modifiers: Vec::new(),
        action:    Action::Quit,
    };
    assert!(keymaps.add_config_mappings(&[mapping]).is_ok());

    let mapping1 = MappingDefinition {
        key_char:  None,
        key_name:  Some(String::from("plus")),
        modifiers: vec![String::from("control")],
        action:    Action::ScrollToBottom,
    };
    let mapping2 = MappingDefinition {
        key_char:  None,
        key_name:  Some(String::from("minus")),
        modifiers: vec![String::from("control"), String::from("shift"), String::from("alt")],
        action:    Action::ScrollToTop,
    };
    assert!(keymaps.add_config_mappings(&[mapping1, mapping2]).is_ok());

    let action = keymaps.get_action(ModifierType::empty(), Key::from_unicode('q'));
    assert_eq!(action, Action::Quit);

    let action = keymaps.get_action(ModifierType::CONTROL_MASK, Key::from_unicode('+'));
    assert_eq!(action, Action::ScrollToBottom);

    let action = keymaps.get_action(
        ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK | ModifierType::MOD1_MASK,
        Key::from_unicode('-')
    );
    assert_eq!(action, Action::ScrollToTop);
}

#[test]
fn test_invalid_mapping_from_config() {
    let mut keymaps = Keymaps::new();

    let mapping = MappingDefinition {
        key_char:  None,
        key_name:  None,
        modifiers: Vec::new(),
        action:    Action::Quit,
    };
    assert!(keymaps.add_config_mappings(&[mapping]).is_err());

    let mapping = MappingDefinition {
        key_char:  Some('q'),
        key_name:  Some(String::from("q")),
        modifiers: Vec::new(),
        action:    Action::Quit,
    };
    assert!(keymaps.add_config_mappings(&[mapping]).is_err());
}
