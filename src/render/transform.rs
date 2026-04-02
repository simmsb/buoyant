use crate::{
    primitives::{Interpolate, transform::LinearTransform},
    render::{AnimatedJoin, AnimationDomain, ContentShape, IntrinsicShape, Render},
};

use super::Diffable;

/// Applies the provided linear transform to the inner render tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transform<T> {
    pub inner: T,
    pub transform: LinearTransform,
}

impl<T> Transform<T> {
    #[must_use]
    pub fn new(inner: T, transform: LinearTransform) -> Self {
        Self { inner, transform }
    }
}

impl<T: Diffable> Diffable for Transform<T> {
    const SIZE: usize = 1 + T::SIZE;

    fn diff_with(&self, other: &Self, differ: &mut super::Differ<'_>) -> bool {
        let mut changed = self.transform != other.transform;

        let r = differ.reserve();

        changed |= if !changed {
            self.inner.diff_with(&other.inner, differ)
        } else {
            differ.push_repeated(true, T::SIZE);
            true
        };

        differ.commit(r, changed);
        changed
    }
}

impl<T: AnimatedJoin> AnimatedJoin for Transform<T> {
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain) {
        self.transform = LinearTransform::interpolate(
            source.transform.clone(),
            self.transform.clone(),
            domain.factor,
        );
        self.inner.join_from(&source.inner, domain);
    }
}

impl<T: Render<C>, C: Interpolate + Copy> Render<C> for Transform<T> {
    fn render(
        &self,
        render_target: &mut impl crate::render_target::RenderTarget<ColorFormat = C>,
        style: &C,
    ) {
        render_target.with_layer(
            |l| l.transform(&self.transform),
            |target| {
                self.inner.render(target, style);
            },
        );
    }

    fn render_animated(
        render_target: &mut impl crate::render_target::RenderTarget<ColorFormat = C>,
        source: &Self,
        target: &Self,
        style: &C,
        domain: &AnimationDomain,
    ) {
        let transform = LinearTransform::interpolate(
            source.transform.clone(),
            target.transform.clone(),
            domain.factor,
        );
        render_target.with_layer(
            |l| l.transform(&transform),
            |render_target| {
                Render::render_animated(render_target, &source.inner, &target.inner, style, domain);
            },
        );
    }

    fn render_animated_diffed(
        render_target: &mut impl crate::render_target::RenderTarget<ColorFormat = C>,
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
            let transform = LinearTransform::interpolate(
                source.transform.clone(),
                target.transform.clone(),
                domain.factor,
            );
            render_target.with_layer(
                |l| l.transform(&transform),
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

impl<T: IntrinsicShape> IntrinsicShape for Transform<T> {
    fn content_shape(&self) -> ContentShape {
        self.inner
            .content_shape()
            .with_offset(self.transform.offset)
    }
}
