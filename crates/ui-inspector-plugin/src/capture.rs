//! Native window matching, capture, and CSS-to-bitmap coordinate conversion.

use image::RgbaImage;
use tauri::{Runtime, WebviewWindow};
use tauri_ui_inspector_core::{
    CoordinateTransform, CssRect, PhysicalPoint, PhysicalRect, PhysicalSize, PixelRect,
    ViewportInfo, crop_image,
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum CaptureError {
    #[error("Tauri window geometry could not be read: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("native window capture failed: {0}")]
    Xcap(#[from] xcap::XCapError),
    #[error("no native window matched the Tauri process and window title")]
    WindowNotFound,
    #[error(transparent)]
    Crop(#[from] tauri_ui_inspector_core::CropError),
}

#[derive(Debug)]
pub(crate) struct CaptureOutput {
    pub(crate) window: RgbaImage,
    pub(crate) element: RgbaImage,
    pub(crate) pixel_crop: PixelRect,
}

pub(crate) fn capture<R: Runtime>(
    window: &WebviewWindow<R>,
    rect: CssRect,
    viewport: ViewportInfo,
    padding: u32,
) -> Result<CaptureOutput, CaptureError> {
    let outer_position = window.outer_position()?;
    let inner_position = window.inner_position()?;
    let outer_size = window.outer_size()?;
    let inner_size = window.inner_size()?;
    let title = window.title()?;
    let native = find_window(
        &title,
        outer_position.x,
        outer_position.y,
        outer_size.width,
        outer_size.height,
    )?;
    let bounds = PhysicalRect {
        origin: PhysicalPoint {
            x: f64::from(native.x()?),
            y: f64::from(native.y()?),
        },
        size: PhysicalSize {
            width: f64::from(native.width()?),
            height: f64::from(native.height()?),
        },
    };
    let image = native.capture_image()?;

    let tauri_to_native_x = bounds.size.width / f64::from(outer_size.width);
    let tauri_to_native_y = bounds.size.height / f64::from(outer_size.height);
    let content_origin = PhysicalPoint {
        x: bounds.origin.x + f64::from(inner_position.x - outer_position.x) * tauri_to_native_x,
        y: bounds.origin.y + f64::from(inner_position.y - outer_position.y) * tauri_to_native_y,
    };
    let content_size = PhysicalSize {
        width: f64::from(inner_size.width) * tauri_to_native_x,
        height: f64::from(inner_size.height) * tauri_to_native_y,
    };
    let transform = CoordinateTransform {
        window_bounds: bounds,
        capture_size: PhysicalSize {
            width: f64::from(image.width()),
            height: f64::from(image.height()),
        },
        content_origin,
        content_size,
        viewport_size: viewport.size,
        visual_viewport_offset: viewport
            .visual_viewport
            .map(|viewport| PhysicalPoint {
                x: viewport.offset_left,
                y: viewport.offset_top,
            })
            .unwrap_or_default(),
    }
    .calibrate_content_from_viewport(viewport.device_pixel_ratio)?;
    let pixel_crop = transform.crop_rect(rect, f64::from(padding))?;
    let element = crop_image(&image, pixel_crop);
    Ok(CaptureOutput {
        window: image,
        element,
        pixel_crop,
    })
}

fn find_window(
    title: &str,
    outer_x: i32,
    outer_y: i32,
    outer_width: u32,
    outer_height: u32,
) -> Result<xcap::Window, CaptureError> {
    let pid = std::process::id();
    let mut matches = xcap::Window::all()?
        .into_iter()
        .filter(|candidate| {
            candidate
                .pid()
                .is_ok_and(|candidate_pid| candidate_pid == pid)
        })
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return Err(CaptureError::WindowNotFound);
    }
    matches.sort_by_key(|candidate| {
        window_match_score(
            candidate.title().ok().as_deref(),
            candidate.x().unwrap_or_default(),
            candidate.y().unwrap_or_default(),
            candidate.width().unwrap_or_default(),
            candidate.height().unwrap_or_default(),
            title,
            outer_x,
            outer_y,
            outer_width,
            outer_height,
        )
    });
    Ok(matches.remove(0))
}

#[allow(
    clippy::too_many_arguments,
    reason = "pure score compares one measured window tuple"
)]
fn window_match_score(
    candidate_title: Option<&str>,
    candidate_x: i32,
    candidate_y: i32,
    candidate_width: u32,
    candidate_height: u32,
    title: &str,
    outer_x: i32,
    outer_y: i32,
    outer_width: u32,
    outer_height: u32,
) -> u64 {
    u64::from(candidate_title != Some(title)) * 1_000_000
        + u64::from(candidate_x.abs_diff(outer_x))
        + u64::from(candidate_y.abs_diff(outer_y))
        + u64::from(candidate_width.abs_diff(outer_width))
        + u64::from(candidate_height.abs_diff(outer_height))
}

#[cfg(test)]
mod tests {
    use super::window_match_score;

    #[test]
    fn position_disambiguates_equal_title_and_size() {
        let exact = window_match_score(
            Some("Inspector"),
            -1200,
            40,
            1280,
            800,
            "Inspector",
            -1200,
            40,
            1280,
            800,
        );
        let other_window = window_match_score(
            Some("Inspector"),
            200,
            40,
            1280,
            800,
            "Inspector",
            -1200,
            40,
            1280,
            800,
        );
        assert!(exact < other_window);
    }
}
