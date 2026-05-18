use crate::{
    primitives::{Interpolate, Point, Size, geometry::Rectangle},
    render::{Animate, AnimatedJoin, Capsule, ContentShape, IntrinsicShape, Offset, Render},
    render_target::RenderTarget,
};

use super::Diffable;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ScrollDragging {
    Dragging,
    NotDragging
}

// This hacks together scroll functionality from existing primitives, but
// a bespoke implementation will eventually replace it
type ScrolInner<T> = Offset<Animate<(Offset<T>, Option<Capsule>, Option<Capsule>), ScrollDragging>>;

/// This is just a metadata structure that allows [`ScrollView`] to mutate its offset and scroll bars
/// without recomputing a new view tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollRenderable<T> {
    pub(crate) scroll_size: Size,
    pub(crate) inner_size: Size,
    pub(crate) inner: ScrolInner<T>,
}

impl<T> ScrollRenderable<T> {
    pub(crate) fn new(scroll_size: Size, inner_size: Size, inner: ScrolInner<T>) -> Self {
        Self {
            scroll_size,
            inner_size,
            inner,
        }
    }

    pub fn offset(&self) -> Point {
        self.inner.subtree.subtree.0.offset
    }

    pub fn offset_mut(&mut self) -> &mut Point {
        &mut self.inner.subtree.subtree.0.offset
    }

    /// The bounds of the scrollview itself
    pub fn bounds(&self) -> Rectangle {
        Rectangle::new(self.inner.offset, self.scroll_size)
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner.subtree.subtree.0.subtree
    }

    pub(crate) fn set_bars(&mut self, horizontal: Option<Capsule>, vertical: Option<Capsule>) {
        self.inner.subtree.subtree.1 = horizontal;
        self.inner.subtree.subtree.2 = vertical;
    }
}

impl<T: Diffable + IntrinsicShape> Diffable for ScrollRenderable<T> {
    const SIZE: usize = 1 + T::SIZE;

    fn diff_with(&self, other: &Self, differ: &mut super::Differ<'_>) {
        let changed = self.scroll_size != other.scroll_size
            || self.inner_size != other.inner_size
            || differ.is_region_dirty(self)
            || differ.is_region_overdrawn(self)
            ;

        let r = differ.reserve();


        if changed {
            differ.push_repeated(true, T::SIZE);
            differ.dirty_aabb_self(other);
            differ.drawn_aabb_self(self);
        } else {
            self.inner.diff_with(&other.inner, differ);
        }

        differ.commit(r, changed || differ.is_region_dirty(self));
    }
}

impl<T: AnimatedJoin> AnimatedJoin for ScrollRenderable<T> {
    fn join_from(&mut self, source: &Self, domain: &crate::render::AnimationDomain) {
        self.scroll_size = Size::interpolate(source.scroll_size, self.scroll_size, domain.factor);
        self.inner_size = Size::interpolate(source.inner_size, self.inner_size, domain.factor);
        self.inner.join_from(&source.inner, domain);
    }
}

impl<T: Render<C>, C: Interpolate + Copy> Render<C> for ScrollRenderable<T> {
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = C>, style: &C) {
        render_target.with_layer(
            |l| l.clip(&Rectangle::new(self.inner.offset, self.scroll_size)),
            |target| {
                self.inner.render(target, style);
            },
        );
    }

    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = C>,
        source: &Self,
        target: &Self,
        style: &C,
        domain: &crate::render::AnimationDomain,
    ) {
        render_target.with_layer(
            |l| l.clip(&Rectangle::new(target.inner.offset, target.scroll_size)),
            |render_target| {
                Render::render_animated(render_target, &source.inner, &target.inner, style, domain);
            },
        );
    }

    fn render_animated_diffed(
        render_target: &mut impl RenderTarget<ColorFormat = C>,
        source: &Self,
        target: &Self,
        style: &C,
        domain: &super::AnimationDomain,
        differ: &mut super::Differ<'_>,
    ) {
        if differ.pop() {
            target.stamp_background(render_target);
            Self::render_animated(render_target, source, target, style, domain);
            differ.ignore(T::SIZE);
        } else {
            render_target.with_layer(
                |l| l.clip(&Rectangle::new(target.inner.offset, target.scroll_size)),
                |render_target| {
                    Render::render_animated_diffed(
                        render_target,
                        &source.inner,
                        &target.inner,
                        style,
                        domain,
                        differ,
                    );
                },
            );
        }
    }
}

impl<T: IntrinsicShape> IntrinsicShape for ScrollRenderable<T> {
    fn content_shape(&self) -> ContentShape {
        self.bounds().into()
    }
}
