use crate::render_target::RenderTarget;

use super::{AnimatedJoin, AnimationDomain, ContentShape, Diffable, IntrinsicShape, Render};

impl Diffable for () {
    const SIZE: usize = 0;

    fn diff_with(&self, _other: &Self, _differ: &mut super::Differ<'_>) -> bool {
        false
    }
}

impl AnimatedJoin for () {
    fn join_from(&mut self, _source: &Self, _domain: &AnimationDomain) {}
}

impl<C> Render<C> for () {
    fn render(&self, _render_target: &mut impl RenderTarget<ColorFormat = C>, _style: &C) {}

    fn render_animated(
        _render_target: &mut impl RenderTarget<ColorFormat = C>,
        _source: &Self,
        _target: &Self,
        _style: &C,
        _domain: &AnimationDomain,
    ) {
    }

    fn render_animated_diffed(
        _render_target: &mut impl RenderTarget<ColorFormat = C>,
        _source: &Self,
        _target: &Self,
        _style: &C,
        _domain: &AnimationDomain,
        _differ: &mut super::Differ<'_>,
    ) {
    }
}

impl IntrinsicShape for () {
    fn content_shape(&self) -> ContentShape {
        ContentShape::Empty
    }
}
