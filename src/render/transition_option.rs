use crate::{
    primitives::{Interpolate, Size},
    render::{AnimatedJoin, ContentShape, IntrinsicShape, Render},
    render_target::RenderTarget,
    transition::{Direction, Transition},
};

use super::{AnimationDomain, Diffable};

/// An optional subtree that can be rendered with a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionOption<Subtree, T> {
    Some {
        subtree: Subtree,
        /// Size of the subtree, used for computing offsets
        size: Size,
        /// The transition to apply
        transition: T,
    },
    None,
}

impl<Subtree, T> TransitionOption<Subtree, T> {
    /// Constructs a new [`Self::Some`] variant
    #[must_use]
    pub const fn new_some(subtree: Subtree, size: Size, transition: T) -> Self {
        Self::Some {
            subtree,
            size,
            transition,
        }
    }
}

impl<Subtree: Diffable, T: Transition + PartialEq> Diffable for TransitionOption<Subtree, T> {
    const SIZE: usize = 1 + Subtree::SIZE;

    fn diff_with(&self, other: &Self, differ: &mut super::Differ<'_>) {
        match (self, other) {
            (
                Self::Some {
                    subtree: this_subtree,
                    size: this_size,
                    transition: this_transition,
                },
                Self::Some {
                    subtree: other_subtree,
                    size: other_size,
                    transition: other_transition,
                },
            ) => {
                let changed = this_size != other_size
                    || this_transition != other_transition
                    || differ.is_region_dirty(self);
                let r = differ.reserve();

                let offset = this_transition.transform(Direction::Out, 0, *this_size);
                let transform = differ.offset(offset);

                if changed {
                    differ.push_repeated(true, Subtree::SIZE);
                    differ.dirty_aabb_self(other);
                    differ.drawn_aabb_self(self);
                } else {
                    this_subtree.diff_with(other_subtree, differ);
                }

                differ.restore_transform(transform);

                differ.commit(r, changed || differ.is_region_dirty(self));
            }
            (Self::None, Self::None) => {
                differ.push(false);
                differ.push_repeated(false, Subtree::SIZE);
            }
            _ => {
                differ.push(true);
                differ.push_repeated(true, Subtree::SIZE);
                differ.dirty_aabb_self(other);
                differ.drawn_aabb_self(self);
            }
        }
    }
}

impl<Subtree: AnimatedJoin + Clone, T: Transition> AnimatedJoin for TransitionOption<Subtree, T> {
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain) {
        if let (
            Self::Some {
                subtree: source_subtree,
                ..
            },
            Self::Some {
                subtree: target_subtree,
                ..
            },
        ) = (source, self)
        {
            target_subtree.join_from(source_subtree, domain);
        }
    }
}

impl<Subtree: Render<Color> + Clone, T: Transition + PartialEq, Color: Interpolate + Copy>
    Render<Color> for TransitionOption<Subtree, T>
{
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = Color>, style: &Color) {
        if let Self::Some { subtree, .. } = self {
            subtree.render(render_target, style);
        }
    }

    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
    ) {
        match (source, target) {
            (
                Self::Some {
                    subtree: source, ..
                },
                Self::Some {
                    subtree: target, ..
                },
            ) => {
                Subtree::render_animated(render_target, source, target, style, domain);
            }
            (
                Self::Some {
                    subtree: source_subtree,
                    size,
                    transition,
                    ..
                },
                Self::None,
            ) => {
                if !domain.is_complete() {
                    let opacity = transition.opacity(Direction::Out, domain.factor);
                    let offset = transition.transform(Direction::Out, domain.factor, *size);
                    render_target.with_layer(
                        |l| l.offset(offset).opacity(opacity),
                        |render_target| {
                            source_subtree.render(render_target, style);
                        },
                    );
                }
            }
            (
                Self::None,
                Self::Some {
                    subtree: target_subtree,
                    size,
                    transition,
                    ..
                },
            ) => {
                if domain.is_complete() {
                    target_subtree.render(render_target, style);
                } else {
                    let opacity = transition.opacity(Direction::In, domain.factor);
                    let offset = transition.transform(Direction::In, domain.factor, *size);
                    render_target.with_layer(
                        |l| l.offset(offset).opacity(opacity),
                        |render_target| {
                            target_subtree.render(render_target, style);
                        },
                    );
                }
            }
            (Self::None, Self::None) => {}
        }
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
            differ.ignore(Subtree::SIZE);
        } else {
            match (source, target) {
                (
                    Self::Some {
                        subtree: source, ..
                    },
                    Self::Some {
                        subtree: target, ..
                    },
                ) => {
                    Subtree::render_animated_diffed(
                        render_target,
                        source,
                        target,
                        style,
                        domain,
                        differ,
                    );
                }
                (
                    Self::Some {
                        subtree: source_subtree,
                        size,
                        transition,
                        ..
                    },
                    Self::None,
                ) => {
                    if !domain.is_complete() {
                        let opacity = transition.opacity(Direction::Out, domain.factor);
                        let offset = transition.transform(Direction::Out, domain.factor, *size);
                        render_target.with_layer(
                            |l| l.offset(offset).opacity(opacity),
                            |render_target| {
                                source_subtree.render(render_target, style);
                            },
                        );
                    }
                    differ.ignore(Subtree::SIZE);
                }
                (
                    Self::None,
                    Self::Some {
                        subtree: target_subtree,
                        size,
                        transition,
                        ..
                    },
                ) => {
                    if domain.is_complete() {
                        target_subtree.render(render_target, style);
                    } else {
                        let opacity = transition.opacity(Direction::In, domain.factor);
                        let offset = transition.transform(Direction::In, domain.factor, *size);
                        render_target.with_layer(
                            |l| l.offset(offset).opacity(opacity),
                            |render_target| {
                                target_subtree.render(render_target, style);
                            },
                        );
                    }

                    differ.ignore(Subtree::SIZE);
                }
                (Self::None, Self::None) => {
                    differ.ignore(Subtree::SIZE);
                }
            }
        }
    }
}

impl<Subtree: IntrinsicShape, T> IntrinsicShape for TransitionOption<Subtree, T> {
    fn content_shape(&self) -> ContentShape {
        match self {
            Self::Some { subtree, .. } => subtree.content_shape(),
            Self::None => ContentShape::Empty,
        }
    }
}
