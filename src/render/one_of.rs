use crate::render::{ContentShape, IntrinsicShape, Render, RenderTarget};

use super::{AnimatedJoin, Diffable};

macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

macro_rules! define_branch {
    ($name:ident, $($variant:ident),+) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name<$($variant),+> {
            $(
                $variant($variant),
            )+
        }

        impl<$($variant),+>  Diffable for $name<$($variant),+>
            where $($variant: Diffable,)+
        {
            const SIZE: usize = max!($($variant::SIZE),+);

            fn diff_with(&self, other: &Self, differ: &mut crate::render::Differ<'_>) {
                match (self, other) {
                    $(
                        (Self::$variant(source), Self::$variant(target)) => {
                            source.diff_with(target, differ);
                            differ.push_repeated(false, Self::SIZE - $variant::SIZE);
                        },
                    )+
                    (_, _) => {
                        differ.push_repeated(true, Self::SIZE);
                        differ.dirty_aabb_self(other);
                        differ.drawn_aabb_self(self);
                    },
                }
            }
        }

        impl<$($variant),+>  AnimatedJoin for $name<$($variant),+>
            where $($variant: AnimatedJoin,)+
        {
            fn join_from(&mut self, source: &Self, domain: &crate::render::AnimationDomain) {
                match (source, self) {
                    $(
                        (Self::$variant(source), Self::$variant(target)) => {
                            target.join_from(source, domain)
                        },
                    )+
                    (_, _) => (),
                }
            }
        }

        impl<C, $($variant),+> Render<C> for $name<$($variant),+>
            where $($variant: Render<C>,)+
        {
            fn render(&self, target: &mut impl RenderTarget<ColorFormat = C>, color: &C) {
                match self {
                    $(
                        Self::$variant(v) => v.render(target, color),
                    )+
                }
            }

            fn render_animated(
                render_target: &mut impl RenderTarget<ColorFormat = C>,
                source: &Self,
                target: &Self,
                style: &C,
                domain: &crate::render::AnimationDomain,
            ) {
                match (source, target) {
                    $(
                        (Self::$variant(source), Self::$variant(target)) => {
                            $variant::render_animated(render_target, source, target, style, domain);
                        },
                    )+
                    (_, target) => {
                        target.render(render_target, style);
                    }
                }
            }

            fn render_animated_diffed(
                render_target: &mut impl RenderTarget<ColorFormat = C>,
                source: &Self,
                target: &Self,
                style: &C,
                domain: &crate::render::AnimationDomain,
                differ: &mut crate::render::Differ<'_>,
            ) {
                match (source, target) {
                    $(
                        (Self::$variant(source), Self::$variant(target)) => {
                            $variant::render_animated_diffed(render_target, source, target, style, domain, differ);
                            differ.ignore(Self::SIZE - $variant::SIZE);
                        },
                    )+
                    (_, target) => {
                        target.render(render_target, style);
                        differ.ignore(Self::SIZE);
                    }
                }
            }
        }

        impl<$($variant),+> IntrinsicShape for $name<$($variant),+>
            where $($variant: IntrinsicShape,)+
        {
            fn content_shape(&self) -> ContentShape {
                match self {
                    $(
                        Self::$variant(v) => v.content_shape(),
                    )+
                }
            }
        }
    }
}

// OneOf1 has no reason to exist, skip it
// Views with only one variant should just use the inner type directly
define_branch!(OneOf2, V0, V1);
define_branch!(OneOf3, V0, V1, V2);
define_branch!(OneOf4, V0, V1, V2, V3);
define_branch!(OneOf5, V0, V1, V2, V3, V4);
define_branch!(OneOf6, V0, V1, V2, V3, V4, V5);
define_branch!(OneOf7, V0, V1, V2, V3, V4, V5, V6);
define_branch!(OneOf8, V0, V1, V2, V3, V4, V5, V6, V7);
define_branch!(OneOf9, V0, V1, V2, V3, V4, V5, V6, V7, V8);
define_branch!(OneOf10, V0, V1, V2, V3, V4, V5, V6, V7, V8, V9);
