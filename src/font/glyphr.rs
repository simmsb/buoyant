use embedded_graphics::{pixelcolor::Rgb888, prelude::PixelColor};

use crate::primitives::{Interpolate, Point, Size, geometry::Rectangle};

use super::{Font, FontMetrics, FontRender};
use crate::font::{self};

struct GlyphrMetrics<'a> {
    font: &'a glyphr::Font<'a>,
    scale: u8,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Scale(pub u8);

impl Default for Scale {
    fn default() -> Self {
        Self(1)
    }
}

impl Interpolate for Scale {
    fn interpolate(from: Self, to: Self, amount: u8) -> Self {
        Self(Interpolate::interpolate(from.0, to.0, amount))
    }
}

impl font::CustomSize for Scale {
    fn with_size(mut self, size: u32) -> Self {
        self.0 = size as u8;
        self
    }
}

impl Font for glyphr::Font<'_> {
    type Attributes = Scale;

    fn metrics(&self, attributes: &Self::Attributes) -> impl super::FontMetrics {
        GlyphrMetrics {
            font: self,
            scale: attributes.0,
        }
    }
}

impl FontMetrics for GlyphrMetrics<'_> {
    fn rendered_size(&self, character: char) -> Option<crate::primitives::geometry::Rectangle> {
        let Ok(glyph) = self.font.find_glyph(character) else {
            defmt::trace!("No glyph found for: {}", character);
            return None;
        };
        defmt::trace!(
            "glyph found for: {}, {}x{}",
            character,
            glyph.width,
            glyph.height
        );

        let scale = self.scale as i32;

        let y_offset = self.font.descent as i32
            + (self.font.ascent as i32 - glyph.ymin as i32 - glyph.height as i32);

        let region = Rectangle::new(
            Point::new(scale * glyph.xmin as i32, scale * y_offset),
            Size::new(
                (scale * glyph.width as i32) as u32,
                (scale * glyph.height as i32) as u32,
            ),
        );

        defmt::trace!("Rendered size of {} found to be {}", character, region);

        Some(region)
    }

    fn vertical_metrics(&self) -> super::VMetrics {
        defmt::trace!(
            "Fetching vertical metrics: ascent: {}, descent: {}, gap: {}",
            self.font.ascent,
            self.font.descent,
            self.font.line_gap,
        );

        let scale = self.scale as i32;

        super::VMetrics {
            ascent: scale * self.font.ascent as i32,
            descent: scale * self.font.descent as i32,
            line_spacing: scale * self.font.line_gap as i32,
        }
    }

    fn advance(&self, character: char) -> u32 {
        let Ok(glyph) = self.font.find_glyph(character) else {
            defmt::trace!("(advance) No glyph found for: {}", character);
            return 0;
        };
        defmt::trace!(
            "(advance) glyph found for: {}, {}",
            character,
            glyph.advance_width
        );

        self.scale as u32 * glyph.advance_width as u32
    }
}

impl font::Sealed for glyphr::Font<'_> {}

impl<C> FontRender<C> for glyphr::Font<'static>
where
    C: PixelColor + Interpolate + core::fmt::Debug + Into<Rgb888> + From<Rgb888>,
{
    fn draw(
        &self,
        character: char,
        offset: Point,
        color: C,
        background_color: Option<C>,
        attributes: &Self::Attributes,
        surface: &mut impl crate::render_target::Surface<Color = C>,
    ) {
        let background = background_color.unwrap_or(Rgb888::default().into());

        let glyphr = glyphr::Glyphr::new();

        let Ok(glyph) = self.find_glyph(character) else {
            return;
        };

        defmt::trace!(
            "Drawing character {} at {},{} with size {}x{}",
            character,
            offset.x,
            offset.y,
            glyph.width,
            glyph.height,
        );

        let scale = attributes.0;

        if scale == 1 {
            let Ok(pixels) = glyphr.pixels(character, *self) else {
                return;
            };

            let y_offset = self.ascent as i32 - glyph.ymin as i32 - glyph.height as i32;

            let region = Rectangle::new(
                offset + Point::new(glyph.xmin as i32, y_offset),
                Size::new(glyph.width as u32, glyph.height as u32),
            );

            surface.fill_contiguous(
                &region,
                pixels.map(|p| {
                    let scaled = p.as_u8() * 16;

                    Interpolate::interpolate(background, color, scaled)
                }),
            );
        } else {
            let Ok(pixels) = glyphr.pixels_scaled(character, *self, scale) else {
                return;
            };

            let y_offset = self.ascent as i32 - glyph.ymin as i32 - glyph.height as i32;

            let region = Rectangle::new(
                offset + Point::new(scale as i32 * glyph.xmin as i32, scale as i32 * y_offset),
                Size::new(
                    scale as u32 * glyph.width as u32,
                    scale as u32 * glyph.height as u32,
                ),
            );

            surface.fill_contiguous(
                &region,
                pixels.map(|p| {
                    let scaled = p.as_u8() * 16;

                    Interpolate::interpolate(background, color, scaled)
                }),
            );
        }
    }
}
