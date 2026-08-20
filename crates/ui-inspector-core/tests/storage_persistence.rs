//! Reference storage integration tests.

#![warn(rust_2018_idioms)]

mod support;

use image::RgbaImage;
use tauri_ui_inspector_core::{Storage, StorageConfig};

#[test]
fn save_writes_json_and_both_pngs_atomically() {
    let temp = tempfile::tempdir().unwrap();
    let storage = Storage::new(StorageConfig {
        root: temp.path().join(".ui-inspector"),
        max_history: 100,
    });
    let reference = support::reference("ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    let image = RgbaImage::new(4, 3);
    let directory = storage
        .save(&reference, Some(&image), Some(&image))
        .unwrap();
    assert!(directory.join("reference.json").is_file());
    assert!(directory.join("window.png").is_file());
    assert!(directory.join("element.png").is_file());
    assert_eq!(storage.get(&reference.id).unwrap().reference(), &reference);
}

#[test]
fn bounded_history_removes_the_oldest_ulid_directory() {
    let temp = tempfile::tempdir().unwrap();
    let storage = Storage::new(StorageConfig {
        root: temp.path().join(".ui-inspector"),
        max_history: 2,
    });
    for id in [
        "ui_01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "ui_01BRZ3NDEKTSV4RRFFQ69G5FAV",
        "ui_01CRZ3NDEKTSV4RRFFQ69G5FAV",
    ] {
        storage.save(&support::reference(id), None, None).unwrap();
    }
    let ids = storage
        .list()
        .unwrap()
        .into_iter()
        .map(|entry| entry.reference().id.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        [
            "ui_01CRZ3NDEKTSV4RRFFQ69G5FAV",
            "ui_01BRZ3NDEKTSV4RRFFQ69G5FAV"
        ]
    );
}
