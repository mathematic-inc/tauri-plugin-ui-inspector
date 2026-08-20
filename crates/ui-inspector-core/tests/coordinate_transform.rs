//! Coordinate transform integration tests.

#![warn(rust_2018_idioms)]

use tauri_ui_inspector_core::{
    CoordinateTransform, CssRect, CssSize, PhysicalPoint, PhysicalRect, PhysicalSize, PixelRect,
};

fn rect(left: f64, top: f64, width: f64, height: f64) -> CssRect {
    CssRect {
        x: left,
        y: top,
        width,
        height,
        top,
        right: left + width,
        bottom: top + height,
        left,
    }
}

fn transform() -> CoordinateTransform {
    CoordinateTransform {
        window_bounds: PhysicalRect {
            origin: PhysicalPoint { x: -200.0, y: 50.0 },
            size: PhysicalSize {
                width: 600.0,
                height: 500.0,
            },
        },
        capture_size: PhysicalSize {
            width: 1200.0,
            height: 1000.0,
        },
        content_origin: PhysicalPoint { x: -200.0, y: 80.0 },
        content_size: PhysicalSize {
            width: 600.0,
            height: 470.0,
        },
        viewport_size: CssSize {
            width: 600.0,
            height: 470.0,
        },
        visual_viewport_offset: PhysicalPoint::default(),
    }
}

#[test]
fn hidpi_and_window_decoration_offsets_are_applied() {
    assert_eq!(
        transform()
            .crop_rect(rect(10.0, 20.0, 100.0, 40.0), 0.0)
            .unwrap(),
        PixelRect {
            x: 20,
            y: 100,
            width: 200,
            height: 80,
        }
    );
}

#[test]
fn browser_viewport_recovers_unreported_macos_title_bar() {
    let mut value = transform();
    value.window_bounds = PhysicalRect {
        origin: PhysicalPoint {
            x: 202.0,
            y: -1270.0,
        },
        size: PhysicalSize {
            width: 1280.0,
            height: 800.0,
        },
    };
    value.capture_size = value.window_bounds.size;
    value.content_origin = value.window_bounds.origin;
    value.content_size = value.window_bounds.size;
    value.viewport_size = CssSize {
        width: 1280.0,
        height: 768.0,
    };

    let value = value.calibrate_content_from_viewport(1.0).unwrap();
    assert_eq!(
        value.crop_rect(rect(16.0, 31.0, 184.0, 40.0), 8.0).unwrap(),
        PixelRect {
            x: 8,
            y: 55,
            width: 200,
            height: 56,
        }
    );
}

#[test]
fn viewport_calibration_accounts_for_hidpi_capture_pixels() {
    let value = transform().calibrate_content_from_viewport(2.0).unwrap();
    assert_eq!(value.content_origin, PhysicalPoint { x: -200.0, y: 80.0 });
    assert_eq!(
        value.content_size,
        PhysicalSize {
            width: 600.0,
            height: 470.0,
        }
    );
}

#[test]
fn padding_is_in_css_pixels_before_scaling() {
    assert_eq!(
        transform()
            .crop_rect(rect(10.0, 20.0, 100.0, 40.0), 8.0)
            .unwrap(),
        PixelRect {
            x: 4,
            y: 84,
            width: 232,
            height: 112,
        }
    );
}

#[test]
fn page_zoom_uses_measured_content_to_viewport_ratio() {
    let mut value = transform();
    value.viewport_size.width = 300.0;
    value.viewport_size.height = 235.0;
    assert_eq!(
        value.crop_rect(rect(10.0, 10.0, 50.0, 20.0), 0.0).unwrap(),
        PixelRect {
            x: 40,
            y: 100,
            width: 200,
            height: 80,
        }
    );
}

#[test]
fn partially_offscreen_elements_are_clamped() {
    assert_eq!(
        transform()
            .crop_rect(rect(-30.0, -50.0, 60.0, 80.0), 0.0)
            .unwrap(),
        PixelRect {
            x: 0,
            y: 0,
            width: 60,
            height: 120,
        }
    );
}

#[test]
fn elements_outside_the_capture_fail() {
    assert!(
        transform()
            .crop_rect(rect(900.0, 900.0, 20.0, 20.0), 0.0)
            .is_err()
    );
}
