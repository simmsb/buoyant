//! Render primitives.
//!
//! If you are constructing views, this is not the module you want. Use types under [`crate::view`] instead.
//!
//! Render primitives represent the minimal information required to render a view. Nodes in the
//! render tree can be animated and joined with other trees of the same type to enable complex
//! animations.

use core::time::Duration;

use crate::primitives::{Point, aabb::StaticAABBTree};
use crate::primitives::{
    geometry::{self, Rectangle},
    transform::{CoordinateSpaceTransform as _, ScaleFactor},
};
use crate::render_target::RenderTarget;
use crate::{
    primitives::{geometry::Shape, transform::LinearTransform},
    render_target::SolidBrush,
};

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
pub(crate) use scroll_renderable::ScrollDragging;
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
    idx: u16,
    pub(crate) array: &'a mut bitvec::slice::BitSlice<u8>,
    pub(crate) dirty_aabb: &'a mut StaticAABBTree<30>,
    pub(crate) drawn_aabb: &'a mut StaticAABBTree<30>,
    transform: LinearTransform,
}

#[derive(Debug)]
pub struct DifferReservation(u16);

#[derive(Debug)]
pub struct DifferGranularity(bool);

#[derive(Debug, Clone, Copy)]
pub struct DifferNote(u16);

impl<'a> Differ<'a> {
    pub fn new(
        array: &'a mut bitvec::slice::BitSlice<u8>,
        dirty_aabb: &'a mut StaticAABBTree<30>,
        drawn_aabb: &'a mut StaticAABBTree<30>,
    ) -> Self {
        Self {
            granular: true,
            idx: 0,
            array,
            dirty_aabb,
            drawn_aabb,
            transform: LinearTransform::default(),
        }
    }

    pub fn become_nongranular(&mut self) -> DifferGranularity {
        let r = DifferGranularity(self.granular);
        self.granular = false;
        r
    }

    pub fn restore_granularity(&mut self, granularity: DifferGranularity) {
        self.granular = granularity.0;
    }
    pub fn restore_transform(&mut self, transform: LinearTransform) {
        self.transform = transform;
    }

    pub fn transform(&mut self, transform: &LinearTransform) -> LinearTransform {
        let r_transform = self.transform.clone();
        self.transform = self.transform.applying(transform);
        r_transform
    }

    pub fn offset(&mut self, offset: Point) -> LinearTransform {
        let transform = self.transform.clone();
        self.transform.offset.x += (offset.x * self.transform.scale.cast_signed()).to_num::<i16>();
        self.transform.offset.y += (offset.y * self.transform.scale.cast_signed()).to_num::<i16>();
        transform
    }

    pub fn scale(&mut self, scale: ScaleFactor) -> LinearTransform {
        let transform = self.transform.clone();
        self.transform.scale *= scale;
        transform
    }

    pub fn is_region_dirty<R: IntrinsicShape>(&self, renderable: &R) -> bool {
        let Some(bb) = renderable.content_shape().bounding_box() else {
            return false;
        };

        let bbt = bb.applying(&self.transform);

        self.test_dirty_region(&bbt)
    }

    pub fn is_region_overdrawn<R: IntrinsicShape>(&self, renderable: &R) -> bool {
        let Some(bb) = renderable.content_shape().bounding_box() else {
            return false;
        };

        let bbt = bb.applying(&self.transform);

        self.test_drawn_region(&bbt)
    }

    pub fn dirty_aabb_self<R: IntrinsicShape>(&mut self, renderable: &R) {
        let Some(bb) = renderable.content_shape().bounding_box() else {
            return;
        };

        let bbt = bb.applying(&self.transform);

        self.add_dirty_region(bbt);
    }

    pub fn drawn_aabb_self<R: IntrinsicShape>(&mut self, renderable: &R) {
        let Some(bb) = renderable.content_shape().bounding_box() else {
            return;
        };

        let bbt = bb.applying(&self.transform);

        self.add_drawn_region(bbt);
    }

    #[must_use]
    pub fn test_dirty_region(&self, region: &Rectangle) -> bool {
        let mut result = false;
        self.dirty_aabb.query_intersects(region, |_| result = true);
        result
    }

    #[must_use]
    pub fn test_drawn_region(&self, region: &Rectangle) -> bool {
        let mut result = false;
        self.drawn_aabb.query_intersects(region, |_| result = true);
        result
    }

    pub fn add_dirty_region(&mut self, region: Rectangle) {
        let mut whole = region.clone();

        self.dirty_aabb
            .drain_contained(&region, |r| whole = whole.union(&r));

        self.dirty_aabb.insert(whole);
    }

    pub fn copy_drawn_as_dirty(&mut self, region: Rectangle) {
        self.drawn_aabb.query_intersects(&region, |r| {
            // println!("Dirtying drawn region {}", r);
            self.dirty_aabb.insert(r.clone());
        });
    }

    pub fn clear_dirty_where_drawn(&mut self, region: Rectangle) {
        self.drawn_aabb.query_overlaps(&region, |r| {
            // println!("Undirtying drawn region {}", r);
            self.dirty_aabb.drain_contained(r, |_| {});
        });
    }

    pub fn add_drawn_region(&mut self, region: Rectangle) {
        let mut whole = region.clone();

        self.dirty_aabb.drain_contained(&region, |_r| {});

        self.drawn_aabb
            .drain_contained(&region, |r| whole = whole.union(&r));

        self.drawn_aabb.insert(whole);
    }

    pub fn reserve(&mut self) -> DifferReservation {
        if !self.granular {
            return DifferReservation(self.idx);
        }

        let idx = self.idx;
        self.idx += 1;

        DifferReservation(idx)
    }

    pub fn note(&mut self) -> DifferNote {
        DifferNote(self.idx)
    }

    pub fn restore(&mut self, note: DifferNote) {
        self.idx = note.0;
    }

    pub fn commit(&mut self, reservation: DifferReservation, changed: bool) {
        if !changed {
            return;
        }

        self.array.set(reservation.0 as usize, changed);
    }

    pub fn push_inner(&mut self, changed: bool) {
        if !self.granular && changed {
            self.array.set(0, changed);
            return;
        }

        let idx = self.idx;
        self.idx += 1;

        if !changed {
            return;
        }

        self.array.set(idx as usize, changed);
    }

    pub fn push(&mut self, changed: bool) {
        self.push_inner(changed);
    }

    pub fn push_repeated(&mut self, changed: bool, n: usize) {
        for _ in 0..n {
            self.push_inner(changed);
        }
    }

    pub fn pop(&mut self) -> bool {
        if !self.granular {
            return self.array[0];
        }

        let idx = self.idx;
        self.idx += 1;

        self.array[idx as usize]
    }

    pub fn ignore(&mut self, n: usize) {
        // println!("Differ ignore {} +{}", self.idx, n);
        self.idx += n as u16;
    }

    pub fn reset(&mut self) {
        self.idx = 0;
    }
}

pub trait Diffable: IntrinsicShape {
    /// The number of differ slots needed for this node and all of its children.
    const SIZE: usize;

    /// Compute the diff of this node in comparison with another.
    ///
    /// Whether this node changed or not should be written to the `differ`. This
    /// function is also responsible for calling `diff_with` on any of its
    /// children.
    ///
    /// After diffing children, if any child was invalidated by dirty regions
    /// contained within this node's bounding box, the contained dirty regions
    /// should be removed and this node should be marked as invalidated.
    fn diff_with(&self, other: &Self, differ: &mut Differ<'_>);
}

pub trait AnimatedJoin {
    /// Modifies a target tree by joining it with the source tree
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain);
}

/// A type that can be rendered to a target and animated
pub trait Render<Color>: AnimatedJoin + IntrinsicShape + Diffable
where
    Color: Copy,
{
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

    fn stamp_background(&self, render_target: &mut impl RenderTarget<ColorFormat = Color>) {
        if let Some(shape) = self.content_shape().bounding_box() {
            let background = render_target.background();
            render_target.fill(
                LinearTransform::default(),
                &SolidBrush::new(background),
                None,
                &shape,
            );
        }
    }

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
