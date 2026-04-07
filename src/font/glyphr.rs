use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{PixelColor, RgbColor as _},
};

use crate::primitives::{Interpolate, Pixel, Point, Size, geometry::Rectangle};

use super::{Font, FontMetrics, FontRender};
use crate::font::{self, CustomSize};

struct GlyphrMetrics<'a> {
    font: &'a glyphr::Font<'a>,
}

impl<'a> Font for glyphr::Font<'a> {
    type Attributes = ();

    fn metrics(&self, attributes: &Self::Attributes) -> impl super::FontMetrics {
        GlyphrMetrics { font: self }
    }
}

impl<'a> FontMetrics for GlyphrMetrics<'a> {
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

        Some(Rectangle::new(
            Point::new(glyph.xmin as i32, glyph.ymin as i32),
            Size::new(glyph.width as u32, glyph.height as u32),
        ))
    }

    fn vertical_metrics(&self) -> super::VMetrics {
        defmt::trace!("Fetching vertical metrics");

        super::VMetrics {
            ascent: self.font.ascent as i32,
            descent: self.font.descent as i32,
            line_spacing: 2,
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

        glyph.advance_width as u32
    }
}

impl font::Sealed for glyphr::Font<'_> {}

// struct RunBuffer {
//     buf: heapless::Vec<Rgb888, 32, u8>,
//     start: Point,
// }

// struct GlyphrRenderProxy<SURFACE> {
//     buf: RunBuffer,
//     background: Rgb888,
//     surface: SURFACE,
// }

// impl<C, SURFACE> GlyphrRenderProxy<SURFACE>
// where
//     C: From<Rgb888>,
//     SURFACE: crate::render_target::Surface<Color = C>,
// {
//     fn flush(&mut self) {
//         if self.buf.buf.is_empty() {
//             return;
//         }

//         let area = Rectangle::new(self.buf.start, Size::new(self.buf.buf.len() as u32, 1));

//         self.surface
//             .fill_contiguous(&area, self.buf.buf.drain(..).map(Into::into));
//     }
// }

// impl<C, SURFACE> glyphr::RenderTarget for GlyphrRenderProxy<SURFACE>
// where
//     C: From<Rgb888>,
//     SURFACE: crate::render_target::Surface<Color = C>,
// {
//     fn write_pixel(&mut self, x: u32, y: u32, color: u32) -> bool {
//         let [a, r, g, b] = color.to_be_bytes();
//         let c = Rgb888::new(r, g, b);
//         let blended = Interpolate::interpolate(self.background, c, a);
//         // let blended = c;

//         if self.buf.buf.is_full()
//             || (self.buf.start + Point::new(self.buf.buf.len() as i32, 0))
//                 != Point::new(x as i32, y as i32)
//         {
//             self.flush();
//         }

//         if self.buf.buf.is_empty() {
//             self.buf.start = Point::new(x as i32, y as i32);
//         }

//         _ = self.buf.buf.push(blended);

//         true
//     }

//     fn dimensions(&self) -> (u32, u32) {
//         let s = self.surface.size();

//         (s.width, s.height)
//     }
// }

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
        _attributes: &Self::Attributes,
        surface: &mut impl crate::render_target::Surface<Color = C>,
    ) {
        let background = background_color.unwrap_or(Rgb888::default().into());
        // let c = color.into();
        // let color = u32::from_be_bytes([0, c.r(), c.g(), c.b()]);

        let glyphr = glyphr::Glyphr::new();

        // let mut render_proxy = GlyphrRenderProxy {
        //     buf: RunBuffer {
        //         buf: heapless::Vec::new(),
        //         start: Point::zero(),
        //     },
        //     background: background_color.map(Into::into).unwrap_or_default(),
        //     surface,
        // };

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

        let Ok(pixels) = glyphr.pixels(character, *self) else {
            return;
        };

        let y_offset = self.descent as i32 + (self.ascent as i32 - glyph.ymin as i32 - glyph.height as i32);

        let region = Rectangle::new(
            offset + Point::new(glyph.xmin as i32, y_offset as i32),
            Size::new(glyph.width as u32, glyph.height as u32),
        );

        surface.fill_contiguous(
            &region,
            pixels.map(|p| {
                let scaled = u8::from(p) * 16;

                Interpolate::interpolate(background, color, scaled)
            }),
        )

        // _ = glyphr.render_char(
        //     &mut render_proxy,
        //     character,
        //     *self,
        //     offset.x,
        //     offset.y,
        // );

        // render_proxy.flush();
    }
}
