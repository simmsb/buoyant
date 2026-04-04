use crate::{
    primitives::Interpolate,
    render::{AnimationDomain, ContentShape, IntrinsicShape, Render, RenderTarget},
};

use super::{AnimatedJoin, Diffable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintBackground<T, C> {
    pub subtree: T,
    pub color: C,
}

impl<T, C> HintBackground<T, C> {
    pub const fn new(subtree: T, color: C) -> Self {
        Self { subtree, color }
    }
}

impl<T: Diffable, C: PartialEq> Diffable for HintBackground<T, C> {
    const SIZE: usize = 1 + T::SIZE;

    fn diff_with(&self, other: &Self, differ: &mut super::Differ<'_>) {
        let changed = self.color != other.color
            || differ.is_region_dirty(self)
            ;

        let r = differ.reserve();
        differ.commit(r, changed);

        if !changed {
            self.subtree.diff_with(&other.subtree, differ);
        } else {
            differ.push_repeated(true, T::SIZE);
            differ.dirty_aabb_self(other);
            differ.drawn_aabb_self(self);
        }

    }
}

impl<T: AnimatedJoin, C: Interpolate + Copy> AnimatedJoin for HintBackground<T, C> {
    fn join_from(&mut self, source: &Self, config: &AnimationDomain) {
        self.color = Interpolate::interpolate(source.color, self.color, config.factor);
        self.subtree.join_from(&source.subtree, config);
    }
}

impl<T: Render<C>, C: Interpolate + Copy> Render<C> for HintBackground<T, C> {
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = C>, style: &C) {
        let color = self.color;
        render_target.with_layer(
            |l| l.hint_background(color),
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
        domain: &AnimationDomain,
    ) {
        let color = Interpolate::interpolate(source.color, target.color, domain.factor);
        render_target.with_layer(
            |l| l.hint_background(color),
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
            Self::render_animated(render_target, source, target, style, domain);
            differ.ignore(T::SIZE);
        } else {
            let color = Interpolate::interpolate(source.color, target.color, domain.factor);
            render_target.with_layer(
                |l| l.hint_background(color),
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

impl<T: IntrinsicShape, C> IntrinsicShape for HintBackground<T, C> {
    fn content_shape(&self) -> ContentShape {
        self.subtree.content_shape()
    }
}
