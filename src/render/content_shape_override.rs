use crate::render::{AnimatedJoin, AnimationDomain, ContentShape, IntrinsicShape, Render};
use crate::render_target::RenderTarget;

use super::Diffable;

/// A render tree node that overrides the content shape of its subtree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentShapeOverride<T> {
    pub subtree: T,
    pub shape: ContentShape,
}

impl<T> ContentShapeOverride<T> {
    pub const fn new(subtree: T, shape: ContentShape) -> Self {
        Self { subtree, shape }
    }
}

impl<T: Diffable> Diffable for ContentShapeOverride<T> {
    const SIZE: usize = 1 + T::SIZE;

    fn diff_with(&self, other: &Self, differ: &mut super::Differ<'_>) {
        let changed = self.shape != other.shape;

        let r = differ.reserve();

        if changed {
            differ.push_repeated(true, T::SIZE);
        } else {
            self.subtree.diff_with(&other.subtree, differ);
        }

        differ.commit(r, changed);
    }
}

impl<T: AnimatedJoin> AnimatedJoin for ContentShapeOverride<T> {
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain) {
        self.subtree.join_from(&source.subtree, domain);
    }
}

impl<T: Render<Color>, Color> Render<Color> for ContentShapeOverride<T> {
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = Color>, style: &Color) {
        self.subtree.render(render_target, style);
    }

    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
    ) {
        T::render_animated(
            render_target,
            &source.subtree,
            &target.subtree,
            style,
            domain,
        );
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
            T::render_animated_diffed(render_target, &source.subtree, &target.subtree, style, domain, differ);
        }
    }
}

impl<T: IntrinsicShape> IntrinsicShape for ContentShapeOverride<T> {
    fn content_shape(&self) -> ContentShape {
        self.shape.clone()
    }
}
