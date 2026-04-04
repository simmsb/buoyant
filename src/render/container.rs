use crate::{
    primitives::{Interpolate, geometry::Rectangle},
    render::{AnimatedJoin, ContentShape, IntrinsicShape, Render},
    render_target::RenderTarget,
};

use super::{AnimationDomain, Diffable};

/// A node that tracks a frame and contains a child view.
///
/// This is used to track view frames in event handlers, where only the child frame
/// would otherwise be available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container<T> {
    pub frame: Rectangle,
    pub child: T,
}

impl<T> Container<T> {
    pub const fn new(frame: Rectangle, child: T) -> Self {
        Self { frame, child }
    }
}

impl<T: Diffable + IntrinsicShape> Diffable for Container<T> {
    const SIZE: usize = 1 + T::SIZE;

    fn diff_with(&self, other: &Self, differ: &mut super::Differ<'_>) {
        let changed = self.frame != other.frame
            || differ.is_region_dirty_or_drawn(self)
            ;

        let r = differ.reserve();

        if !changed {
            self.child.diff_with(&other.child, differ);
        } else {
            differ.push_repeated(true, T::SIZE);
            differ.dirty_aabb_self(other);
            differ.drawn_aabb_self(self);
        };

        differ.commit(r, changed || differ.is_region_dirty(self));
    }
}

impl<T: AnimatedJoin> AnimatedJoin for Container<T> {
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain) {
        self.frame =
            Rectangle::interpolate(source.frame.clone(), self.frame.clone(), domain.factor);
        self.child.join_from(&source.child, domain);
    }
}

impl<T: Render<Color>, Color> Render<Color> for Container<T> {
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = Color>, style: &Color) {
        self.child.render(render_target, style);
    }

    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
    ) {
        T::render_animated(render_target, &source.child, &target.child, style, domain);
    }

    fn render_animated_diffed(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
        differ: &mut super::Differ<'_>,
    ) {
        if differ.pop() {
            Self::render_animated(render_target, source, target, style, domain);
            differ.ignore(T::SIZE);
        } else {
            T::render_animated_diffed(render_target, &source.child, &target.child, style, domain, differ);
        }
    }
}

impl<T: IntrinsicShape> IntrinsicShape for Container<T> {
    fn content_shape(&self) -> ContentShape {
        self.child.content_shape()
    }
}
