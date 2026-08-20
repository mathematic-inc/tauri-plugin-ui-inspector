//! Reference identifier integration tests.

#![warn(rust_2018_idioms)]

use tauri_ui_inspector_core::ReferenceId;

#[test]
fn mention_and_plain_forms_normalize_to_the_same_id() {
    let plain = ReferenceId::parse("ui_01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
    let mention = ReferenceId::parse("@ui_01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
    assert_eq!(plain, mention);
    assert_eq!(plain.mention(), "@ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
}

#[test]
fn invalid_values_are_rejected() {
    assert!(ReferenceId::parse("ui_not-a-ulid").is_err());
    assert!(ReferenceId::parse("01ARZ3NDEKTSV4RRFFQ69G5FAV").is_err());
}
