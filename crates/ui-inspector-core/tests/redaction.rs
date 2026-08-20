//! Reference redaction integration tests.

#![warn(rust_2018_idioms)]

mod support;

use tauri_ui_inspector_core::{
    DomAncestor, Locator, LocatorStrategy, RedactionConfig, redact_reference,
};

#[test]
fn password_values_are_removed_regardless_of_configuration() {
    let mut reference = support::reference("ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    reference.element.accessibility.input_type = Some("password".to_owned());
    reference.element.accessibility.value = Some("hunter2".to_owned());
    reference
        .element
        .attributes
        .insert("type".to_owned(), "password".to_owned());
    reference
        .element
        .attributes
        .insert("value".to_owned(), "hunter2".to_owned());
    redact_reference(&mut reference, &RedactionConfig::new());
    assert_eq!(reference.element.accessibility.value, None);
    assert_eq!(reference.element.attributes["value"], "[redacted]");
    assert_eq!(reference.element.text, None);
    assert_eq!(reference.element.attributes["type"], "password");
    assert_eq!(reference.dom.html, "[redacted]");
}

#[test]
fn token_like_text_controls_do_not_retain_opted_in_values() {
    let mut reference = support::reference("ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    reference.element.accessibility.input_type = Some("text".to_owned());
    reference.element.accessibility.value = Some("sk-secret".to_owned());
    reference
        .element
        .attributes
        .insert("name".to_owned(), "api-token".to_owned());
    redact_reference(&mut reference, &RedactionConfig::new());
    assert_eq!(reference.element.accessibility.value, None);
}

#[test]
fn configured_text_redaction_removes_agent_visible_copy() {
    let mut reference = support::reference("ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    let mut config = RedactionConfig::new();
    reference.element.accessibility.name = Some("Save".to_owned());
    reference.element.accessibility.description = Some("Save changes".to_owned());
    reference.element.accessibility.aria_label = Some("Save".to_owned());
    reference.element.selectors.preferred = Some("button[name=\"Save\"]".to_owned());
    reference.element.selectors.text = Some("Save".to_owned());
    reference.element.locators = vec![
        Locator {
            strategy: LocatorStrategy::Role,
            value: "button".to_owned(),
            attribute: None,
            name: Some("Save".to_owned()),
            confidence: 0.95,
            unique: true,
        },
        Locator {
            strategy: LocatorStrategy::Text,
            value: "Save".to_owned(),
            attribute: None,
            name: None,
            confidence: 0.25,
            unique: true,
        },
    ];
    reference.dom.ancestry.push(DomAncestor {
        tag_name: "main".to_owned(),
        id: None,
        classes: Vec::new(),
        role: Some("main".to_owned()),
        accessible_name: Some("Workspace".to_owned()),
    });
    config.redact_text = true;
    redact_reference(&mut reference, &config);
    assert_eq!(reference.element.text, None);
    assert_eq!(reference.element.accessible_name, None);
    assert_eq!(reference.element.accessibility.aria_label, None);
    assert_eq!(reference.element.selectors.preferred, None);
    assert_eq!(reference.element.selectors.text, None);
    assert_eq!(reference.element.locators.len(), 1);
    assert_eq!(reference.element.locators[0].name, None);
    assert_eq!(reference.dom.ancestry[0].accessible_name, None);
    assert_eq!(reference.dom.html, "[redacted]");
    assert!(!reference.summary.contains("Save"));
}
