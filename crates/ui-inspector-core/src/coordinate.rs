//! Coordinate conversion from CSS viewport units to native screenshot pixels.
//!
//! The transform deliberately uses measured viewport, content-area, window,
//! and capture dimensions instead of assuming any two pixel spaces are equal.

use image::RgbaImage;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::CropError;

/// A size in CSS pixels.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CssSize {
    /// Width in CSS pixels.
    pub width: f64,
    /// Height in CSS pixels.
    pub height: f64,
}

/// An element rectangle in CSS viewport coordinates.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CssRect {
    /// Alias of `left`, retained for `DOMRect` compatibility.
    pub x: f64,
    /// Alias of `top`, retained for `DOMRect` compatibility.
    pub y: f64,
    /// Width in CSS pixels.
    pub width: f64,
    /// Height in CSS pixels.
    pub height: f64,
    /// Top edge in CSS viewport coordinates.
    pub top: f64,
    /// Right edge in CSS viewport coordinates.
    pub right: f64,
    /// Bottom edge in CSS viewport coordinates.
    pub bottom: f64,
    /// Left edge in CSS viewport coordinates.
    pub left: f64,
}

/// A point in native screen coordinates.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalPoint {
    /// Horizontal coordinate.
    pub x: f64,
    /// Vertical coordinate.
    pub y: f64,
}

/// A size in native screen coordinates.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalSize {
    /// Width in native units.
    pub width: f64,
    /// Height in native units.
    pub height: f64,
}

/// A rectangle in native screen coordinates.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalRect {
    /// Top-left screen coordinate.
    pub origin: PhysicalPoint,
    /// Native size.
    pub size: PhysicalSize,
}

/// An integer rectangle within a captured image.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PixelRect {
    /// Left pixel in the image.
    pub x: u32,
    /// Top pixel in the image.
    pub y: u32,
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
}

/// Converts CSS element rectangles into captured-image pixel rectangles.
///
/// `window_bounds` is the native window rectangle reported by the capture
/// backend. `content_origin` and `content_size` describe Tauri's webview
/// content area in native screen coordinates. `capture_size` is the actual PNG
/// size, which may be larger than the native bounds on `HiDPI` displays.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CoordinateTransform {
    /// Native window bounds used by the capture backend.
    pub window_bounds: PhysicalRect,
    /// Actual screenshot bitmap size.
    pub capture_size: PhysicalSize,
    /// Native screen position of the webview content area.
    pub content_origin: PhysicalPoint,
    /// Native size of the webview content area.
    pub content_size: PhysicalSize,
    /// Browser viewport size in CSS pixels.
    pub viewport_size: CssSize,
    /// Visual viewport offset used during pinch zoom.
    pub visual_viewport_offset: PhysicalPoint,
}

impl CoordinateTransform {
    /// Calibrates the webview content area from browser viewport metrics.
    ///
    /// Some window backends, notably macOS, report the outer window geometry
    /// for both Tauri's inner and outer geometry APIs. The browser viewport and
    /// device pixel ratio still describe the rendered content exactly, so they
    /// provide a reliable fallback for title-bar and border offsets.
    ///
    /// # Errors
    ///
    /// Returns [`CropError::InvalidTransform`] when any required dimension or
    /// `device_pixel_ratio` is zero or non-finite.
    pub fn calibrate_content_from_viewport(
        mut self,
        device_pixel_ratio: f64,
    ) -> Result<Self, CropError> {
        validate_positive(self.window_bounds.size.width, "window width")?;
        validate_positive(self.window_bounds.size.height, "window height")?;
        validate_positive(self.capture_size.width, "capture width")?;
        validate_positive(self.capture_size.height, "capture height")?;
        validate_positive(self.viewport_size.width, "viewport width")?;
        validate_positive(self.viewport_size.height, "viewport height")?;
        validate_positive(device_pixel_ratio, "device pixel ratio")?;

        let capture_scale_x = self.capture_size.width / self.window_bounds.size.width;
        let capture_scale_y = self.capture_size.height / self.window_bounds.size.height;
        let content_width = (self.viewport_size.width * device_pixel_ratio / capture_scale_x)
            .min(self.window_bounds.size.width);
        let content_height = (self.viewport_size.height * device_pixel_ratio / capture_scale_y)
            .min(self.window_bounds.size.height);
        let horizontal_chrome = self.window_bounds.size.width - content_width;
        let vertical_chrome = self.window_bounds.size.height - content_height;
        let reported_x = self.content_origin.x - self.window_bounds.origin.x;
        let reported_y = self.content_origin.y - self.window_bounds.origin.y;

        if reported_x.abs() < 0.5 && horizontal_chrome > 0.5 {
            self.content_origin.x = self.window_bounds.origin.x + horizontal_chrome / 2.0;
        }
        if reported_y.abs() < 0.5 && vertical_chrome > 0.5 {
            self.content_origin.y = self.window_bounds.origin.y + vertical_chrome;
        }
        self.content_size = PhysicalSize {
            width: content_width,
            height: content_height,
        };
        Ok(self)
    }

    /// Converts an element rectangle to an image crop, clamped to the capture.
    ///
    /// `padding` is expressed in CSS pixels.
    ///
    /// # Errors
    ///
    /// Returns [`CropError::InvalidTransform`] for zero or non-finite
    /// dimensions and [`CropError::OutsideCapture`] when the padded rectangle
    /// does not intersect the captured image.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::{
    ///     CoordinateTransform, CssRect, CssSize, PhysicalPoint, PhysicalRect,
    ///     PhysicalSize, PixelRect,
    /// };
    ///
    /// let transform = CoordinateTransform {
    ///     window_bounds: PhysicalRect {
    ///         origin: PhysicalPoint { x: 10.0, y: 20.0 },
    ///         size: PhysicalSize { width: 500.0, height: 400.0 },
    ///     },
    ///     capture_size: PhysicalSize { width: 1000.0, height: 800.0 },
    ///     content_origin: PhysicalPoint { x: 10.0, y: 40.0 },
    ///     content_size: PhysicalSize { width: 500.0, height: 380.0 },
    ///     viewport_size: CssSize { width: 500.0, height: 380.0 },
    ///     visual_viewport_offset: PhysicalPoint::default(),
    /// };
    /// let rect = CssRect {
    ///     left: 20.0, top: 10.0, right: 120.0, bottom: 50.0,
    ///     x: 20.0, y: 10.0, width: 100.0, height: 40.0,
    /// };
    /// assert_eq!(transform.crop_rect(rect, 0.0).unwrap(), PixelRect {
    ///     x: 40, y: 60, width: 200, height: 80,
    /// });
    /// ```
    pub fn crop_rect(&self, rect: CssRect, padding: f64) -> Result<PixelRect, CropError> {
        validate_positive(self.window_bounds.size.width, "window width")?;
        validate_positive(self.window_bounds.size.height, "window height")?;
        validate_positive(self.capture_size.width, "capture width")?;
        validate_positive(self.capture_size.height, "capture height")?;
        validate_pixel_dimension(self.capture_size.width, "capture width")?;
        validate_pixel_dimension(self.capture_size.height, "capture height")?;
        validate_positive(self.content_size.width, "content width")?;
        validate_positive(self.content_size.height, "content height")?;
        validate_positive(self.viewport_size.width, "viewport width")?;
        validate_positive(self.viewport_size.height, "viewport height")?;

        let content_scale_x = self.content_size.width / self.viewport_size.width;
        let content_scale_y = self.content_size.height / self.viewport_size.height;
        let capture_scale_x = self.capture_size.width / self.window_bounds.size.width;
        let capture_scale_y = self.capture_size.height / self.window_bounds.size.height;
        let padding = padding.max(0.0);

        let css_left = rect.left + self.visual_viewport_offset.x - padding;
        let css_top = rect.top + self.visual_viewport_offset.y - padding;
        let css_right = rect.right + self.visual_viewport_offset.x + padding;
        let css_bottom = rect.bottom + self.visual_viewport_offset.y + padding;

        let left = ((self.content_origin.x + css_left * content_scale_x
            - self.window_bounds.origin.x)
            * capture_scale_x)
            .floor();
        let top = ((self.content_origin.y + css_top * content_scale_y
            - self.window_bounds.origin.y)
            * capture_scale_y)
            .floor();
        let right = ((self.content_origin.x + css_right * content_scale_x
            - self.window_bounds.origin.x)
            * capture_scale_x)
            .ceil();
        let bottom = ((self.content_origin.y + css_bottom * content_scale_y
            - self.window_bounds.origin.y)
            * capture_scale_y)
            .ceil();

        let left = left.clamp(0.0, self.capture_size.width);
        let top = top.clamp(0.0, self.capture_size.height);
        let right = right.clamp(0.0, self.capture_size.width);
        let bottom = bottom.clamp(0.0, self.capture_size.height);

        if right <= left || bottom <= top {
            return Err(CropError::OutsideCapture);
        }

        Ok(PixelRect {
            x: pixel_value(left),
            y: pixel_value(top),
            width: pixel_value(right - left),
            height: pixel_value(bottom - top),
        })
    }
}

fn validate_pixel_dimension(value: f64, name: &'static str) -> Result<(), CropError> {
    if value <= f64::from(u32::MAX) {
        Ok(())
    } else {
        Err(CropError::InvalidTransform(name))
    }
}

fn pixel_value(value: f64) -> u32 {
    debug_assert!(value.is_finite() && value >= 0.0 && value <= f64::from(u32::MAX));
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        value as u32
    }
}

/// Crops a screenshot without copying pixels outside `rect`.
///
/// # Panics
///
/// Panics if `rect` is not wholly contained in `image`.
///
/// # Examples
///
/// ```
/// use image::RgbaImage;
/// use tauri_ui_inspector_core::{PixelRect, crop_image};
///
/// let image = RgbaImage::new(20, 10);
/// let crop = crop_image(&image, PixelRect { x: 2, y: 3, width: 4, height: 5 });
/// assert_eq!(crop.dimensions(), (4, 5));
/// ```
#[track_caller]
#[must_use]
pub fn crop_image(image: &RgbaImage, rect: PixelRect) -> RgbaImage {
    assert!(
        rect.x.saturating_add(rect.width) <= image.width()
            && rect.y.saturating_add(rect.height) <= image.height(),
        "crop rectangle must be contained in the source image"
    );
    image::imageops::crop_imm(image, rect.x, rect.y, rect.width, rect.height).to_image()
}

fn validate_positive(value: f64, name: &'static str) -> Result<(), CropError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(CropError::InvalidTransform(name))
    }
}
