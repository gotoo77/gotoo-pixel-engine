use crate::{
    framebuffer::Framebuffer,
    image::{Image, ImageRegion},
    pixel::Pixel,
    viewport::Rect,
};

/// Controls how a source image is fitted into a destination rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFit {
    /// Preserves aspect ratio as closely as integer raster dimensions allow and fits the
    /// complete source inside the destination. Unused destination pixels are left untouched.
    Contain,
    /// Preserves aspect ratio and fills the complete destination, cropping the source
    /// symmetrically around its center when the aspect ratios differ.
    Cover,
    /// Maps the complete source directly onto the destination without preserving aspect ratio.
    Stretch,
}

/// Controls how source texels are sampled while an image is scaled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFilter {
    /// Selects the nearest source texel without blending neighboring texels.
    Nearest,
    /// Bilinearly interpolates the four neighboring texels with alpha-correct premultiplied
    /// color interpolation before framebuffer blending.
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RasterRect {
    x: i64,
    y: i64,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FitPlan {
    draw: RasterRect,
    x_map: AxisMap,
    y_map: AxisMap,
}

/// Exact affine mapping from destination pixel centers to source texel-center coordinates.
///
/// `coordinate = (slope * (2 * destination_index + 1) + offset) / denominator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AxisMap {
    slope: i128,
    offset: i128,
    denominator: i128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AxisSample {
    low: u32,
    high: u32,
    weight_num: u128,
    weight_den: u128,
}

impl AxisMap {
    fn full(source_len: u32, destination_len: u32) -> Self {
        debug_assert!(source_len > 0);
        debug_assert!(destination_len > 0);
        Self {
            slope: i128::from(source_len),
            offset: -i128::from(destination_len),
            denominator: i128::from(destination_len) * 2,
        }
    }

    fn cover_crop(
        source_len: u32,
        destination_len: u32,
        scale_source_len: u32,
        scale_destination_len: u32,
    ) -> Self {
        debug_assert!(source_len > 0);
        debug_assert!(destination_len > 0);
        debug_assert!(scale_source_len > 0);
        debug_assert!(scale_destination_len > 0);

        let source_len = i128::from(source_len);
        let destination_len = i128::from(destination_len);
        let scale_source_len = i128::from(scale_source_len);
        let scale_destination_len = i128::from(scale_destination_len);

        Self {
            slope: scale_source_len,
            offset: (source_len - 1) * scale_destination_len - destination_len * scale_source_len,
            denominator: scale_destination_len * 2,
        }
    }

    fn coordinate_numerator(self, destination_index: u32) -> i128 {
        self.slope * (i128::from(destination_index) * 2 + 1) + self.offset
    }

    fn nearest_index(self, destination_index: u32, source_len: u32) -> u32 {
        if source_len <= 1 {
            return 0;
        }

        // Nearest texel center is floor(coordinate + 0.5). Keeping the expression
        // rational avoids floating-point boundary drift.
        let edge_numerator = self.coordinate_numerator(destination_index) * 2 + self.denominator;
        if edge_numerator <= 0 {
            return 0;
        }

        let index = edge_numerator / (self.denominator * 2);
        index.min(i128::from(source_len - 1)) as u32
    }

    fn linear_sample(self, destination_index: u32, source_len: u32) -> AxisSample {
        if source_len <= 1 {
            return AxisSample {
                low: 0,
                high: 0,
                weight_num: 0,
                weight_den: 1,
            };
        }

        let numerator = self.coordinate_numerator(destination_index);
        if numerator <= 0 {
            return AxisSample {
                low: 0,
                high: 0,
                weight_num: 0,
                weight_den: 1,
            };
        }

        let last = source_len - 1;
        let last_numerator = i128::from(last) * self.denominator;
        if numerator >= last_numerator {
            return AxisSample {
                low: last,
                high: last,
                weight_num: 0,
                weight_den: 1,
            };
        }

        let low = (numerator / self.denominator) as u32;
        let remainder = (numerator % self.denominator) as u128;
        AxisSample {
            low,
            high: low + 1,
            weight_num: remainder,
            weight_den: self.denominator as u128,
        }
    }
}

impl Framebuffer {
    /// Draws a complete image into `destination` using the selected fitting and filtering modes.
    ///
    /// `Contain` preserves aspect ratio as closely as integer raster dimensions allow and leaves
    /// letterbox/pillarbox pixels untouched. `Cover` preserves aspect ratio and performs a
    /// centered crop. `Stretch` maps the complete source to the complete destination. Drawing is
    /// clipped to the framebuffer.
    pub fn draw_image_fit(
        &mut self,
        image: &Image,
        destination: Rect,
        fit: ImageFit,
        filter: ImageFilter,
    ) {
        self.draw_image_region_fit(
            image,
            ImageRegion::new(0, 0, image.width(), image.height()),
            destination,
            fit,
            filter,
        );
    }

    /// Draws a source image region into `destination` using the selected fitting and filtering modes.
    ///
    /// The source region is clipped to the image before fitting. Empty source or destination
    /// rectangles draw nothing. Sampling uses pixel-center coordinates and never reads outside
    /// the clipped source region.
    pub fn draw_image_region_fit(
        &mut self,
        image: &Image,
        region: ImageRegion,
        destination: Rect,
        fit: ImageFit,
        filter: ImageFilter,
    ) {
        let Some(region) = clipped_source_region(image, region) else {
            return;
        };
        let Some(plan) = fit_plan(region, destination, fit) else {
            return;
        };
        let Some((left, top, right, bottom)) = visible_destination(self, plan.draw) else {
            return;
        };

        for destination_y in top..bottom {
            let local_y = (i64::from(destination_y) - plan.draw.y) as u32;
            for destination_x in left..right {
                let local_x = (i64::from(destination_x) - plan.draw.x) as u32;
                let source = sample_rgba(
                    image, region, plan.x_map, plan.y_map, local_x, local_y, filter,
                );
                blend_scaled_pixel(self, destination_x, destination_y, source);
            }
        }
    }
}

fn clipped_source_region(image: &Image, region: ImageRegion) -> Option<ImageRegion> {
    if image.width() == 0
        || image.height() == 0
        || region.width == 0
        || region.height == 0
        || region.x >= image.width()
        || region.y >= image.height()
    {
        return None;
    }

    let max_x = region.x.saturating_add(region.width).min(image.width());
    let max_y = region.y.saturating_add(region.height).min(image.height());
    if max_x <= region.x || max_y <= region.y {
        return None;
    }

    Some(ImageRegion::new(
        region.x,
        region.y,
        max_x - region.x,
        max_y - region.y,
    ))
}

fn fit_plan(region: ImageRegion, destination: Rect, fit: ImageFit) -> Option<FitPlan> {
    if region.width == 0 || region.height == 0 || destination.width == 0 || destination.height == 0
    {
        return None;
    }

    let source_width = region.width;
    let source_height = region.height;
    let destination_x = i64::from(destination.x);
    let destination_y = i64::from(destination.y);

    match fit {
        ImageFit::Stretch => Some(FitPlan {
            draw: RasterRect {
                x: destination_x,
                y: destination_y,
                width: destination.width,
                height: destination.height,
            },
            x_map: AxisMap::full(source_width, destination.width),
            y_map: AxisMap::full(source_height, destination.height),
        }),
        ImageFit::Contain => {
            let width_limited = u128::from(destination.width) * u128::from(source_height)
                <= u128::from(destination.height) * u128::from(source_width);
            let (draw_width, draw_height) = if width_limited {
                let height = (u128::from(destination.width) * u128::from(source_height)
                    / u128::from(source_width)) as u32;
                (destination.width, height.max(1).min(destination.height))
            } else {
                let width = (u128::from(destination.height) * u128::from(source_width)
                    / u128::from(source_height)) as u32;
                (width.max(1).min(destination.width), destination.height)
            };

            Some(FitPlan {
                draw: RasterRect {
                    x: destination_x + i64::from((destination.width - draw_width) / 2),
                    y: destination_y + i64::from((destination.height - draw_height) / 2),
                    width: draw_width,
                    height: draw_height,
                },
                x_map: AxisMap::full(source_width, draw_width),
                y_map: AxisMap::full(source_height, draw_height),
            })
        }
        ImageFit::Cover => {
            let width_dominates = u128::from(destination.width) * u128::from(source_height)
                >= u128::from(destination.height) * u128::from(source_width);
            let (x_map, y_map) = if width_dominates {
                (
                    AxisMap::full(source_width, destination.width),
                    AxisMap::cover_crop(
                        source_height,
                        destination.height,
                        source_width,
                        destination.width,
                    ),
                )
            } else {
                (
                    AxisMap::cover_crop(
                        source_width,
                        destination.width,
                        source_height,
                        destination.height,
                    ),
                    AxisMap::full(source_height, destination.height),
                )
            };

            Some(FitPlan {
                draw: RasterRect {
                    x: destination_x,
                    y: destination_y,
                    width: destination.width,
                    height: destination.height,
                },
                x_map,
                y_map,
            })
        }
    }
}

fn visible_destination(
    framebuffer: &Framebuffer,
    destination: RasterRect,
) -> Option<(u32, u32, u32, u32)> {
    if framebuffer.width() == 0
        || framebuffer.height() == 0
        || destination.width == 0
        || destination.height == 0
    {
        return None;
    }

    let left = destination.x.max(0);
    let top = destination.y.max(0);
    let right = (destination.x + i64::from(destination.width)).min(i64::from(framebuffer.width()));
    let bottom =
        (destination.y + i64::from(destination.height)).min(i64::from(framebuffer.height()));

    if left >= right || top >= bottom {
        return None;
    }

    Some((left as u32, top as u32, right as u32, bottom as u32))
}

fn sample_rgba(
    image: &Image,
    region: ImageRegion,
    x_map: AxisMap,
    y_map: AxisMap,
    destination_x: u32,
    destination_y: u32,
    filter: ImageFilter,
) -> [u8; 4] {
    match filter {
        ImageFilter::Nearest => {
            let source_x = region.x + x_map.nearest_index(destination_x, region.width);
            let source_y = region.y + y_map.nearest_index(destination_y, region.height);
            image_pixel_rgba(image, source_x, source_y)
        }
        ImageFilter::Linear => {
            let x = x_map.linear_sample(destination_x, region.width);
            let y = y_map.linear_sample(destination_y, region.height);
            let top_left = image_pixel_rgba(image, region.x + x.low, region.y + y.low);
            let top_right = image_pixel_rgba(image, region.x + x.high, region.y + y.low);
            let bottom_left = image_pixel_rgba(image, region.x + x.low, region.y + y.high);
            let bottom_right = image_pixel_rgba(image, region.x + x.high, region.y + y.high);

            bilinear_rgba(top_left, top_right, bottom_left, bottom_right, x, y)
        }
    }
}

fn bilinear_rgba(
    top_left: [u8; 4],
    top_right: [u8; 4],
    bottom_left: [u8; 4],
    bottom_right: [u8; 4],
    x: AxisSample,
    y: AxisSample,
) -> [u8; 4] {
    let x_low = x.weight_den - x.weight_num;
    let y_low = y.weight_den - y.weight_num;
    let denominator = x.weight_den * y.weight_den;
    let weights = [
        x_low * y_low,
        x.weight_num * y_low,
        x_low * y.weight_num,
        x.weight_num * y.weight_num,
    ];
    let pixels = [top_left, top_right, bottom_left, bottom_right];

    let alpha_weighted: u128 = pixels
        .iter()
        .zip(weights.iter())
        .map(|(pixel, &weight)| u128::from(pixel[3]) * weight)
        .sum();
    let alpha = ((alpha_weighted + denominator / 2) / denominator).min(255) as u8;
    if alpha == 0 {
        return [0; 4];
    }

    let mut output = [0; 4];
    for channel in 0..3 {
        let premultiplied_weighted: u128 = pixels
            .iter()
            .zip(weights.iter())
            .map(|(pixel, &weight)| {
                u128::from(pixel[channel]) * u128::from(pixel[3]) * weight
            })
            .sum();
        output[channel] =
            ((premultiplied_weighted + alpha_weighted / 2) / alpha_weighted).min(255) as u8;
    }
    output[3] = alpha;

    output
}

fn image_pixel_rgba(image: &Image, x: u32, y: u32) -> [u8; 4] {
    if x >= image.width() || y >= image.height() {
        return [0; 4];
    }

    let index = (u64::from(y) * u64::from(image.width()) + u64::from(x)) * 4;
    let Ok(index) = usize::try_from(index) else {
        return [0; 4];
    };
    let Some(end) = index.checked_add(4) else {
        return [0; 4];
    };
    let Some(pixel) = image.as_rgba8().get(index..end) else {
        return [0; 4];
    };

    [pixel[0], pixel[1], pixel[2], pixel[3]]
}

fn blend_scaled_pixel(framebuffer: &mut Framebuffer, x: u32, y: u32, source: [u8; 4]) {
    if x >= framebuffer.width() || y >= framebuffer.height() || source[3] == 0 {
        return;
    }
    if source[3] == 255 {
        framebuffer.set_pixel_in_bounds(x, y, Pixel::rgba(source[0], source[1], source[2], 255));
        return;
    }

    let index = (u64::from(y) * u64::from(framebuffer.width()) + u64::from(x)) * 4;
    let Ok(index) = usize::try_from(index) else {
        return;
    };
    let Some(end) = index.checked_add(4) else {
        return;
    };
    let Some(destination) = framebuffer.as_rgba8().get(index..end) else {
        return;
    };
    let destination = [
        destination[0],
        destination[1],
        destination[2],
        destination[3],
    ];

    let source_alpha = u16::from(source[3]);
    let inverse_alpha = 255 - source_alpha;
    let blend_channel = |source: u8, destination: u8| -> u8 {
        ((u16::from(source) * source_alpha + u16::from(destination) * inverse_alpha + 127) / 255)
            as u8
    };
    let output_alpha = source_alpha + ((u16::from(destination[3]) * inverse_alpha + 127) / 255);

    framebuffer.set_pixel_in_bounds(
        x,
        y,
        Pixel::rgba(
            blend_channel(source[0], destination[0]),
            blend_channel(source[1], destination[1]),
            blend_channel(source[2], destination[2]),
            output_alpha.min(255) as u8,
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image_from_pixels(width: u32, height: u32, pixels: &[Pixel]) -> Image {
        let rgba8 = pixels.iter().flat_map(|pixel| pixel.to_rgba8()).collect();
        Image::from_rgba8(width, height, rgba8).expect("test image dimensions must be valid")
    }

    fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn contain_same_aspect_ratio_uses_full_destination() {
        let plan = fit_plan(
            ImageRegion::new(0, 0, 4, 2),
            rect(10, 20, 12, 6),
            ImageFit::Contain,
        )
        .expect("non-empty fit");

        assert_eq!(
            plan.draw,
            RasterRect {
                x: 10,
                y: 20,
                width: 12,
                height: 6,
            }
        );
    }

    #[test]
    fn contain_centers_inside_wider_and_taller_destinations() {
        let wide = fit_plan(
            ImageRegion::new(0, 0, 4, 2),
            rect(10, 20, 12, 12),
            ImageFit::Contain,
        )
        .expect("non-empty fit");
        let tall = fit_plan(
            ImageRegion::new(0, 0, 2, 4),
            rect(10, 20, 12, 12),
            ImageFit::Contain,
        )
        .expect("non-empty fit");

        assert_eq!(
            wide.draw,
            RasterRect {
                x: 10,
                y: 23,
                width: 12,
                height: 6
            }
        );
        assert_eq!(
            tall.draw,
            RasterRect {
                x: 13,
                y: 20,
                width: 6,
                height: 12
            }
        );
    }

    #[test]
    fn cover_keeps_destination_and_centers_source_crop() {
        let horizontal = fit_plan(
            ImageRegion::new(0, 0, 4, 2),
            rect(0, 0, 6, 6),
            ImageFit::Cover,
        )
        .expect("non-empty fit");
        let vertical = fit_plan(
            ImageRegion::new(0, 0, 2, 4),
            rect(0, 0, 6, 6),
            ImageFit::Cover,
        )
        .expect("non-empty fit");

        assert_eq!(
            horizontal.draw,
            RasterRect {
                x: 0,
                y: 0,
                width: 6,
                height: 6
            }
        );
        assert_eq!(horizontal.x_map.nearest_index(0, 4), 1);
        assert_eq!(horizontal.x_map.nearest_index(5, 4), 2);
        assert_eq!(
            vertical.draw,
            RasterRect {
                x: 0,
                y: 0,
                width: 6,
                height: 6
            }
        );
        assert_eq!(vertical.y_map.nearest_index(0, 4), 1);
        assert_eq!(vertical.y_map.nearest_index(5, 4), 2);
    }

    #[test]
    fn contain_leaves_unused_destination_pixels_untouched() {
        let image = image_from_pixels(2, 2, &[Pixel::WHITE; 4]);
        let mut framebuffer = Framebuffer::new(6, 2);
        framebuffer.clear(Pixel::BLACK);

        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 6, 2),
            ImageFit::Contain,
            ImageFilter::Nearest,
        );

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLACK));
        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::BLACK));
        assert_eq!(framebuffer.pixel(2, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(3, 1), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(4, 0), Some(Pixel::BLACK));
        assert_eq!(framebuffer.pixel(5, 1), Some(Pixel::BLACK));
    }

    #[test]
    fn cover_crops_source_around_center() {
        let image = image_from_pixels(4, 1, &[Pixel::RED, Pixel::GREEN, Pixel::BLUE, Pixel::WHITE]);
        let mut framebuffer = Framebuffer::new(2, 2);

        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 2, 2),
            ImageFit::Cover,
            ImageFilter::Nearest,
        );

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::GREEN));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(0, 1), Some(Pixel::GREEN));
        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::BLUE));
    }

    #[test]
    fn nearest_enlargement_repeats_source_texels() {
        let image = image_from_pixels(2, 2, &[Pixel::RED, Pixel::GREEN, Pixel::BLUE, Pixel::WHITE]);
        let mut framebuffer = Framebuffer::new(4, 4);

        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 4, 4),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );

        for y in 0..4 {
            for x in 0..4 {
                let expected = match (x >= 2, y >= 2) {
                    (false, false) => Pixel::RED,
                    (true, false) => Pixel::GREEN,
                    (false, true) => Pixel::BLUE,
                    (true, true) => Pixel::WHITE,
                };
                assert_eq!(framebuffer.pixel(x, y), Some(expected));
            }
        }
    }

    #[test]
    fn nearest_reduction_samples_destination_pixel_centers() {
        let image = image_from_pixels(4, 1, &[Pixel::RED, Pixel::GREEN, Pixel::BLUE, Pixel::WHITE]);
        let mut framebuffer = Framebuffer::new(2, 1);

        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 2, 1),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::GREEN));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::WHITE));
    }

    #[test]
    fn linear_filter_interpolates_middle_pixel() {
        let image = image_from_pixels(2, 1, &[Pixel::BLACK, Pixel::WHITE]);
        let mut framebuffer = Framebuffer::new(3, 1);

        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 3, 1),
            ImageFit::Stretch,
            ImageFilter::Linear,
        );

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLACK));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::rgb(128, 128, 128)));
        assert_eq!(framebuffer.pixel(2, 0), Some(Pixel::WHITE));
    }

    #[test]
    fn linear_transparent_edge_is_alpha_correct() {
        let image = image_from_pixels(
            2,
            1,
            &[Pixel::rgba(0, 0, 0, 0), Pixel::rgba(255, 255, 255, 255)],
        );
        let region = ImageRegion::new(0, 0, 2, 1);
        let plan = fit_plan(region, rect(0, 0, 3, 1), ImageFit::Stretch).expect("non-empty fit");

        assert_eq!(
            sample_rgba(
                &image,
                region,
                plan.x_map,
                plan.y_map,
                1,
                0,
                ImageFilter::Linear,
            ),
            [255, 255, 255, 128]
        );

        let mut framebuffer = Framebuffer::new(3, 1);
        framebuffer.clear(Pixel::BLACK);
        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 3, 1),
            ImageFit::Stretch,
            ImageFilter::Linear,
        );
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::rgb(128, 128, 128)));
    }

    #[test]
    fn linear_fully_transparent_rgb_does_not_bleed() {
        let hidden_black = image_from_pixels(
            2,
            1,
            &[Pixel::rgba(0, 0, 0, 0), Pixel::rgba(0, 255, 0, 255)],
        );
        let hidden_magenta = image_from_pixels(
            2,
            1,
            &[
                Pixel::rgba(255, 0, 255, 0),
                Pixel::rgba(0, 255, 0, 255),
            ],
        );

        let mut first = Framebuffer::new(3, 1);
        let mut second = Framebuffer::new(3, 1);
        first.clear(Pixel::BLACK);
        second.clear(Pixel::BLACK);
        first.draw_image_fit(
            &hidden_black,
            rect(0, 0, 3, 1),
            ImageFit::Stretch,
            ImageFilter::Linear,
        );
        second.draw_image_fit(
            &hidden_magenta,
            rect(0, 0, 3, 1),
            ImageFit::Stretch,
            ImageFilter::Linear,
        );

        assert_eq!(first.as_rgba8(), second.as_rgba8());
        assert_eq!(first.pixel(1, 0), Some(Pixel::rgb(0, 128, 0)));
    }

    #[test]
    fn linear_2d_center_uses_four_texels() {
        let image = image_from_pixels(
            2,
            2,
            &[
                Pixel::rgb(40, 0, 0),
                Pixel::rgb(0, 80, 0),
                Pixel::rgb(0, 0, 120),
                Pixel::rgb(160, 200, 240),
            ],
        );
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 3, 3),
            ImageFit::Stretch,
            ImageFilter::Linear,
        );

        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::rgb(50, 70, 90)));
    }

    #[test]
    fn image_region_fit_scales_only_the_requested_subregion() {
        let image = image_from_pixels(4, 1, &[Pixel::RED, Pixel::GREEN, Pixel::BLUE, Pixel::WHITE]);
        let mut framebuffer = Framebuffer::new(4, 1);

        framebuffer.draw_image_region_fit(
            &image,
            ImageRegion::new(1, 0, 2, 1),
            rect(0, 0, 4, 1),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );

        assert_eq!(
            framebuffer.as_rgba8(),
            &[
                0, 255, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 0, 0, 255, 255,
            ]
        );
    }

    #[test]
    fn source_region_is_clipped_before_scaling_and_empty_region_draws_nothing() {
        let image = image_from_pixels(4, 1, &[Pixel::RED, Pixel::GREEN, Pixel::BLUE, Pixel::WHITE]);
        let mut framebuffer = Framebuffer::new(3, 1);
        framebuffer.clear(Pixel::BLACK);

        framebuffer.draw_image_region_fit(
            &image,
            ImageRegion::new(3, 0, 4, 1),
            rect(0, 0, 2, 1),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );
        framebuffer.draw_image_region_fit(
            &image,
            ImageRegion::new(4, 0, 1, 1),
            rect(2, 0, 1, 1),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(2, 0), Some(Pixel::BLACK));
    }

    #[test]
    fn destination_clipping_preserves_source_mapping() {
        let image = image_from_pixels(2, 1, &[Pixel::RED, Pixel::GREEN]);
        let mut framebuffer = Framebuffer::new(2, 1);
        framebuffer.clear(Pixel::BLACK);

        framebuffer.draw_image_fit(
            &image,
            rect(-1, 0, 2, 1),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::GREEN));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::BLACK));

        framebuffer.draw_image_fit(
            &image,
            rect(1, 0, 2, 1),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::RED));
    }

    #[test]
    fn zero_sized_and_fully_offscreen_destinations_draw_nothing() {
        let image = image_from_pixels(1, 1, &[Pixel::WHITE]);
        let mut framebuffer = Framebuffer::new(2, 2);
        framebuffer.clear(Pixel::BLACK);

        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 0, 2),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );
        framebuffer.draw_image_fit(
            &image,
            rect(i32::MAX, i32::MAX, 10, 10),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );

        assert!(
            framebuffer
                .as_rgba8()
                .chunks_exact(4)
                .all(|pixel| pixel == [0, 0, 0, 255])
        );
    }

    #[test]
    fn extreme_destination_extent_is_clipped_without_coordinate_overflow() {
        let image = image_from_pixels(2, 1, &[Pixel::RED, Pixel::GREEN]);
        let mut framebuffer = Framebuffer::new(2, 1);

        framebuffer.draw_image_fit(
            &image,
            rect(i32::MIN, 0, u32::MAX, 1),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::GREEN));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::GREEN));
    }

    #[test]
    fn scaled_draw_uses_the_existing_source_over_alpha_semantics() {
        let image = image_from_pixels(1, 1, &[Pixel::rgba(200, 100, 50, 128)]);
        let mut framebuffer = Framebuffer::new(1, 1);
        framebuffer.set_pixel_in_bounds(0, 0, Pixel::rgba(20, 40, 60, 80));

        framebuffer.draw_image_fit(
            &image,
            rect(0, 0, 1, 1),
            ImageFit::Stretch,
            ImageFilter::Nearest,
        );

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::rgba(110, 70, 55, 168)));
    }
}
