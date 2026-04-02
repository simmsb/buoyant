//! Render primitives.
//!
//! If you are constructing views, this is not the module you want. Use types under [`crate::view`] instead.
//!
//! Render primitives represent the minimal information required to render a view. Nodes in the
//! render tree can be animated and joined with other trees of the same type to enable complex
//! animations.

use core::time::Duration;

use crate::primitives::Point;
use crate::primitives::geometry;
use crate::primitives::geometry::Shape;
use crate::render_target::RenderTarget;

mod animate;
pub mod chart;
mod clipped;
pub mod collections;
mod container;
mod content_shape_override;
mod empty;
mod hint_background;
#[cfg(feature = "embedded-graphics")]
mod image;
mod offset;
mod one_of;
mod opacity;
mod option;
mod scroll_renderable;
mod shade_subtree;
pub mod shape;
pub mod text;
mod transform;
mod transition_option;

pub use animate::Animate;
pub use clipped::Clipped;
pub use container::Container;
pub use content_shape_override::ContentShapeOverride;
pub use hint_background::HintBackground;
#[cfg(feature = "embedded-graphics")]
pub use image::Image;
pub use offset::Offset;
pub use one_of::{OneOf2, OneOf3, OneOf4, OneOf5, OneOf6, OneOf7, OneOf8, OneOf9, OneOf10};
pub use opacity::Opacity;
pub use scroll_renderable::ScrollRenderable;
pub use shade_subtree::ShadeSubtree;
pub use shape::Capsule;
pub use shape::Circle;
pub use shape::Rect;
pub use shape::RoundedRect;
pub use shape::StrokedShape;
pub use text::Text;
pub use transform::Transform;
pub use transition_option::TransitionOption;

#[derive(Debug)]
pub struct Differ<'a> {
    /// granular is false if we've entered a context where we don't statically
    /// know the size
    granular: bool,
    idx: usize,
    pub(crate) array: &'a mut bitvec::slice::BitSlice<u8>,
    anything_changed: bool,
}

#[derive(Debug)]
pub struct DifferReservation(usize);

#[derive(Debug, Clone, Copy)]
pub struct DifferNote(usize);

impl<'a> Differ<'a> {
    pub fn new(array: &'a mut bitvec::slice::BitSlice<u8>) -> Self {
        Self {
            granular: true,
            idx: 0,
            array,
            anything_changed: false,
        }
    }

    pub fn reset_anything_changed(&mut self) -> bool {
        let anything_changed = self.anything_changed;
        self.anything_changed = false;
        anything_changed
    }

    pub fn restore_anything_changed(&mut self, changed: bool) {
        self.anything_changed |= changed;
    }

    pub fn reserve(&mut self) -> DifferReservation {
        if !self.granular {
            return DifferReservation(self.idx)
        }

        let idx = self.idx;
        self.idx += 1;

        DifferReservation(idx)
    }

    pub fn note(&mut self) -> DifferNote {
        return DifferNote(self.idx)
    }

    pub fn overwrite_from_note(&mut self, note: DifferNote, changed: bool) {
        println!("Overwrite from note {} to {}", note.0, self.idx);
        self.anything_changed |= changed;
        self.array[note.0..self.idx].fill(changed);
    }

    pub fn commit(&mut self, reservation: DifferReservation, changed: bool) {
        self.anything_changed |= changed;
        self.array.set(reservation.0, changed);
    }

    pub fn push(&mut self, changed: bool) {
        self.anything_changed |= changed;
        if !self.granular {
            self.array.set(0, changed);
        }

        let idx = self.idx;
        self.idx += 1;

        self.array.set(idx, changed);
    }

    pub fn push_repeated(&mut self, changed: bool, n: usize) {
        for _ in 0..n {
            self.push(changed);
        }
    }

    pub fn pop(&mut self) -> bool {
        if !self.granular {
            return self.array[0];
        }

        let idx = self.idx;
        self.idx += 1;

        self.array[idx]
    }

    pub fn ignore(&mut self, n: usize) {
        self.idx += n;
    }

    pub fn reset(&mut self) {
        self.idx = 0;
    }

    pub fn any_changed(&self) -> bool {
        self.anything_changed
    }
}

pub trait Diffable {
    /// The number of differ slots needed for this node and all of its children.
    const SIZE: usize;

    /// Compute the diff of this node in comparison with another.
    ///
    /// Whether this node changed or not should be written to the `differ`. This
    /// function is also responsible for calling `diff_with` on any of its
    /// children.
    ///
    /// The return value indicates whether this component changed in some way
    /// which means the parent should also be invalid (e.g. because it moved or
    /// resized).
    ///
    /// Therefore the children should be diffed first, and then the result of
    /// this call ORed with whether the parent changed.
    fn diff_with(&self, other: &Self, differ: &mut Differ<'_>) -> bool;
}

pub trait AnimatedJoin {
    /// Modifies a target tree by joining it with the source tree
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain);
}

/// A type that can be rendered to a target and animated
pub trait Render<Color>: AnimatedJoin + IntrinsicShape + Diffable {
    /// Render the view to the screen
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = Color>, style: &Color);

    /// Render view and all subviews, animating from a source view to a target view
    ///
    /// The implementation of this method should match the implementation of
    /// [`AnimatedJoin::join_from`] to get smooth continuous animations
    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
    );

    fn render_animated_diffed(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
        differ: &mut Differ<'_>,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationDomain {
    /// The animation factor of this domain, ranging from 0 to 255
    pub factor: u8,
    /// The duration since the application started
    pub app_time: Duration,
}

impl AnimationDomain {
    #[must_use]
    pub const fn new(factor: u8, app_time: Duration) -> Self {
        Self { factor, app_time }
    }

    /// Use this to create a new top-level animation domain when rendering
    #[must_use]
    pub const fn top_level(app_time: Duration) -> Self {
        Self {
            factor: 255,
            app_time,
        }
    }

    /// Whether the animation defined by this domain is complete
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.factor == 255
    }
}

/// A type which has an intrinsic content shape
pub trait IntrinsicShape {
    /// The shape of the object, in its local coordinate space
    fn content_shape(&self) -> ContentShape;
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentShape {
    /// The focused element has no content shape
    Empty,
    Rectangle(geometry::Rectangle),
    RoundedRectangle(geometry::RoundedRectangle),
    Circle(geometry::Circle),
}

impl From<geometry::Rectangle> for ContentShape {
    fn from(rect: geometry::Rectangle) -> Self {
        Self::Rectangle(rect)
    }
}

impl From<geometry::RoundedRectangle> for ContentShape {
    fn from(rrect: geometry::RoundedRectangle) -> Self {
        Self::RoundedRectangle(rrect)
    }
}

impl From<geometry::Circle> for ContentShape {
    fn from(circle: geometry::Circle) -> Self {
        Self::Circle(circle)
    }
}

impl ContentShape {
    /// Returns the bounding rectangle of this content shape, if any.
    #[must_use]
    pub fn bounding_box(&self) -> Option<geometry::Rectangle> {
        match self {
            Self::Empty => None,
            Self::Rectangle(rect) => Some(rect.clone()),
            Self::RoundedRectangle(rrect) => Some(rrect.bounding_box()),
            Self::Circle(circle) => Some(circle.bounding_box()),
        }
    }

    /// Returns a new content shape with the provided offset applied
    #[must_use]
    pub fn with_offset(self, offset: Point) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Rectangle(rect) => Self::Rectangle(rect.with_offset(offset)),
            Self::RoundedRectangle(rrect) => Self::RoundedRectangle(rrect.with_offset(offset)),
            Self::Circle(circle) => Self::Circle(circle.with_offset(offset)),
        }
    }

    /// Offsets the shape
    pub fn offset(&mut self, offset: Point) {
        match self {
            Self::Empty => (),
            Self::Rectangle(rect) => rect.offset(offset),
            Self::RoundedRectangle(rrect) => rrect.offset(offset),
            Self::Circle(circle) => circle.offset(offset),
        }
    }

    /// Returns true if the provided point is contained within this content shape
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        // FIXME: implement more precise hit testing for rounded rectangles and circles
        match self {
            Self::Empty => false,
            Self::Rectangle(rect) => rect.contains(&point),
            Self::RoundedRectangle(rrect) => rrect.bounding_box().contains(&point),
            Self::Circle(circle) => circle.bounding_box().contains(&point),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::primitives::{
        Size,
        geometry::{Circle, Rectangle, RoundedRectangle},
    };

    use super::*;

    #[test]
    fn rectangle_bounding_box() {
        let rect = Rectangle {
            origin: Point { x: 10, y: 20 },
            size: Size {
                width: 30,
                height: 40,
            },
        }
        .with_offset(Point { x: 5, y: 5 });

        let bounding_box = rect.bounding_box();
        assert_eq!(
            bounding_box,
            Rectangle {
                origin: Point { x: 15, y: 25 },
                size: Size {
                    width: 30,
                    height: 40
                }
            }
        );
        let content_shape = ContentShape::Rectangle(rect);
        assert_eq!(content_shape.bounding_box(), Some(bounding_box));
    }

    #[test]
    fn circle_bounding_box() {
        let circle = Circle {
            origin: Point { x: 10, y: 20 },
            diameter: 10,
        }
        .with_offset(Point { x: 5, y: 5 });

        let bounding_box = circle.bounding_box();
        assert_eq!(
            bounding_box,
            Rectangle {
                origin: Point { x: 15, y: 25 },
                size: Size {
                    width: 10,
                    height: 10
                }
            }
        );
        let content_shape = ContentShape::Circle(circle);
        assert_eq!(content_shape.bounding_box(), Some(bounding_box));
    }

    #[test]
    fn rounded_rectangle_bounding_box() {
        let rrect = RoundedRectangle {
            origin: Point { x: 10, y: 20 },
            size: Size {
                width: 30,
                height: 40,
            },
            radius: 5,
        }
        .with_offset(Point { x: 5, y: 5 });

        let bounding_box = rrect.bounding_box();
        assert_eq!(
            bounding_box,
            Rectangle {
                origin: Point { x: 15, y: 25 },
                size: Size {
                    width: 30,
                    height: 40
                }
            }
        );
        let content_shape = ContentShape::RoundedRectangle(rrect);
        assert_eq!(content_shape.bounding_box(), Some(bounding_box));
    }

    #[test]
    fn empty_content_shape() {
        let content_shape = ContentShape::Empty;
        assert_eq!(content_shape.bounding_box(), None);
        assert!(!content_shape.contains(Point { x: 0, y: 0 }));
    }
}
