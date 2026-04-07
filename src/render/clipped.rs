use crate::{
    primitives::{Interpolate, geometry::Rectangle},
    render_target::RenderTarget,
};

use super::{AnimatedJoin, AnimationDomain, ContentShape, Diffable, IntrinsicShape, Render};

/// A render tree node that clips its children
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clipped<T> {
    pub subtree: T,
    pub clip_rect: Rectangle,
}

impl<T> Clipped<T> {
    pub const fn new(subtree: T, clip_rect: Rectangle) -> Self {
        Self { subtree, clip_rect }
    }
}

impl<T: Diffable + IntrinsicShape> Diffable for Clipped<T> {
    const SIZE: usize = 1 + T::SIZE;

    fn diff_with(&self, other: &Self, differ: &mut super::Differ<'_>) {
        let changed = self.clip_rect != other.clip_rect
            || differ.is_region_dirty(self)
            ;

        let r = differ.reserve();
        differ.commit(r, changed);

        if changed {
            differ.push_repeated(true, T::SIZE);
            differ.dirty_aabb_self(other);
            differ.drawn_aabb_self(self);
        } else {
            self.subtree.diff_with(&other.subtree, differ);
        }
    }
}

impl<T: AnimatedJoin> AnimatedJoin for Clipped<T> {
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain) {
        self.subtree.join_from(&source.subtree, domain);
        self.clip_rect = Rectangle::interpolate(
            source.clip_rect.clone(),
            self.clip_rect.clone(),
            domain.factor,
        );
    }
}

impl<T: Render<C>, C: Interpolate + Copy> Render<C> for Clipped<T> {
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = C>, style: &C) {
        render_target.with_layer(
            |l| l.clip(&self.clip_rect),
            |render_target| {
                self.subtree.render(render_target, style);
            },
        );
    }

    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = C>,
        source: &Self,
        target: &Self,
        style: &C,
        domain: &super::AnimationDomain,
    ) {
        let clip_rect = Rectangle::interpolate(
            source.clip_rect.clone(),
            target.clip_rect.clone(),
            domain.factor,
        );
        render_target.with_layer(
            |l| l.clip(&clip_rect),
            |render_target| {
                T::render_animated(
                    render_target,
                    &source.subtree,
                    &target.subtree,
                    style,
                    domain,
                );
            },
        );
    }

    fn render_animated_diffed(
        render_target: &mut impl RenderTarget<ColorFormat = C>,
        source: &Self,
        target: &Self,
        style: &C,
        domain: &AnimationDomain,
        differ: &mut super::Differ<'_>,
    ) {
        if differ.pop() {
            target.stamp_background(render_target);
            Self::render_animated(render_target, source, target, style, domain);
            differ.ignore(T::SIZE);
        } else {
            let clip_rect = Rectangle::interpolate(
                source.clip_rect.clone(),
                target.clip_rect.clone(),
                domain.factor,
            );
            render_target.with_layer(
                |l| l.clip(&clip_rect),
                |render_target| {
                    T::render_animated_diffed(
                        render_target,
                        &source.subtree,
                        &target.subtree,
                        style,
                        domain,
                        differ,
                    );
                },
            );
        }
    }
}

impl<T: IntrinsicShape> IntrinsicShape for Clipped<T> {
    fn content_shape(&self) -> ContentShape {
        // FIXME: Clip content shape?
        self.subtree.content_shape()
    }
}
