use crate::primitives::geometry::Rectangle;

use super::{
    AnimatedJoin, AnimationDomain, ContentShape, Diffable, Differ, IntrinsicShape, Render,
    RenderTarget,
};

macro_rules! impl_diffable_for_collections {
    ($(($n:tt, $type:ident)),+) => {
        impl<$($type: crate::render::Diffable + crate::render::IntrinsicShape),+> crate::render::Diffable for ($($type),+) {
            const SIZE: usize = 0 $(+ $type::SIZE)+;

            fn diff_with(&self, other: &Self, differ: &mut crate::render::Differ<'_>) {
                let note = differ.note();

                $({
                    self.$n.diff_with(&other.$n, differ);
                })+

                differ.restore(note);

                // Two passes, as tuples are used for both overlapping and
                // nonoverlapping renderables.

                $({
                    self.$n.diff_with(&other.$n, differ);
                })+
            }
        }
    };
}

#[rustfmt::skip]
mod impl_diffable {
    impl_diffable_for_collections!((0, T0), (1, T1));
    impl_diffable_for_collections!((0, T0), (1, T1), (2, T2));
    impl_diffable_for_collections!((0, T0), (1, T1), (2, T2), (3, T3));
    impl_diffable_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4));
    impl_diffable_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5));
    impl_diffable_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6));
    impl_diffable_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7));
    impl_diffable_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8));
    impl_diffable_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8), (9, T9));
}

macro_rules! impl_join_for_collections {
    ($(($n:tt, $type:ident)),+) => {
        impl<$($type: crate::render::AnimatedJoin),+> crate::render::AnimatedJoin for ($($type),+) {
            fn join_from(
                &mut self,
                source: &Self,
                domain: &crate::render::AnimationDomain
            ) {
                $(
                    self.$n.join_from(
                        &source.$n,
                        domain
                    );
                )+
            }
        }
    };
}

#[rustfmt::skip]
mod impl_join {
    impl_join_for_collections!((0, T0), (1, T1));
    impl_join_for_collections!((0, T0), (1, T1), (2, T2));
    impl_join_for_collections!((0, T0), (1, T1), (2, T2), (3, T3));
    impl_join_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4));
    impl_join_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5));
    impl_join_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6));
    impl_join_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7));
    impl_join_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8));
    impl_join_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8), (9, T9));
}

macro_rules! impl_content_shape_for_collections {
    ($(($n:tt, $type:ident)),+) => {
        impl<$($type: crate::render::IntrinsicShape),+> crate::render::IntrinsicShape for ($($type),+) {
            fn content_shape(&self) -> super::ContentShape {
                let mut result: Option<crate::primitives::geometry::Rectangle> = None;
                $(
                    let shape = self.$n.content_shape();
                    if let Some(bbox) = shape.bounding_box() {
                        result = Some(match result {
                            Some(r) => r.union(&bbox),
                            None => bbox,
                        });
                    }
                )+
                match result {
                    Some(r) => super::ContentShape::Rectangle(r),
                    None => super::ContentShape::Empty,
                }
            }
        }
    };
}

#[rustfmt::skip]
mod impl_content_shape {
    impl_content_shape_for_collections!((0, T0), (1, T1));
    impl_content_shape_for_collections!((0, T0), (1, T1), (2, T2));
    impl_content_shape_for_collections!((0, T0), (1, T1), (2, T2), (3, T3));
    impl_content_shape_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4));
    impl_content_shape_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5));
    impl_content_shape_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6));
    impl_content_shape_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7));
    impl_content_shape_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8));
    impl_content_shape_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8), (9, T9));
}

/// For unsized slices we have no way to know the size, so we have to fall back
/// to whether the entire list changed.
impl<T: Diffable + IntrinsicShape> Diffable for [T] {
    const SIZE: usize = 1;

    fn diff_with(&self, other: &Self, differ: &mut Differ<'_>) {
        differ.granular = false;

        for (a, b) in self.iter().zip(other) {
            a.diff_with(b, differ);
        }

        differ.granular = true;
    }
}

impl<T: AnimatedJoin> AnimatedJoin for [T] {
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain) {
        self.iter_mut().zip(source).for_each(|(target, source)| {
            target.join_from(source, domain);
        });
    }
}

impl<T: Diffable + IntrinsicShape, const N: usize> Diffable for [T; N] {
    const SIZE: usize = T::SIZE * N;

    fn diff_with(&self, other: &Self, differ: &mut Differ<'_>) {
        for (a, b) in self.iter().zip(other) {
            a.diff_with(b, differ);
        }
    }
}

impl<T: Diffable + IntrinsicShape, const N: usize> Diffable for heapless::Vec<T, N> {
    const SIZE: usize = T::SIZE * N;

    fn diff_with(&self, other: &Self, differ: &mut Differ<'_>) {
        for (a, b) in self.iter().zip(other) {
            a.diff_with(b, differ);
        }

        // fill what wasn't compared in the slice with false
        let remainder = N - self.len().min(other.len());

        differ.push_repeated(false, T::SIZE * remainder);
    }
}

impl<T: AnimatedJoin, const N: usize> AnimatedJoin for [T; N] {
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain) {
        self.as_mut_slice().join_from(source.as_slice(), domain);
    }
}

impl<T: AnimatedJoin, const N: usize> AnimatedJoin for heapless::Vec<T, N> {
    fn join_from(&mut self, source: &Self, domain: &AnimationDomain) {
        self.as_mut_slice().join_from(source.as_slice(), domain);
    }
}

impl<T: IntrinsicShape> IntrinsicShape for [T] {
    fn content_shape(&self) -> ContentShape {
        let result = self.iter().fold(None, |acc: Option<Rectangle>, item| {
            let shape = item.content_shape();
            match (acc, shape.bounding_box()) {
                (Some(acc), Some(bbox)) => Some(acc.union(&bbox)),
                (None, Some(bbox)) => Some(bbox),
                (acc, None) => acc,
            }
        });
        result.map_or(ContentShape::Empty, ContentShape::Rectangle)
    }
}

impl<T: IntrinsicShape, const N: usize> IntrinsicShape for [T; N] {
    fn content_shape(&self) -> ContentShape {
        self.as_slice().content_shape()
    }
}

impl<T: IntrinsicShape, const N: usize> IntrinsicShape for heapless::Vec<T, N> {
    fn content_shape(&self) -> ContentShape {
        self.as_slice().content_shape()
    }
}

macro_rules! impl_render_for_collections {
    ($(($n:tt, $type:ident)),+) => {
        impl<Color: Copy, $($type: crate::render::Render<Color> ),+> crate::render::Render<Color> for ($($type),+) {
            fn render(
                &self,
                target: &mut impl crate::render_target::RenderTarget<ColorFormat = Color>,
                style: &Color,
            ) {
                $(
                    self.$n.render(target, style);
                )+
            }

            fn render_animated(
                render_target: &mut impl crate::render_target::RenderTarget<ColorFormat = Color>,
                source: &Self,
                target: &Self,
                style: &Color,
                domain: &crate::render::AnimationDomain,
            ) {
                $(
                    $type::render_animated(
                        render_target,
                        &source.$n,
                        &target.$n,
                        style,
                        domain,
                    );
                )+
            }

            fn render_animated_diffed(
                render_target: &mut impl crate::render_target::RenderTarget<ColorFormat = Color>,
                source: &Self,
                target: &Self,
                style: &Color,
                domain: &crate::render::AnimationDomain,
                differ: &mut crate::render::Differ<'_>
            ) {
                $(
                    $type::render_animated_diffed(
                        render_target,
                        &source.$n,
                        &target.$n,
                        style,
                        domain,
                        differ
                    );
                )+
            }
        }
    };
}

#[rustfmt::skip]
mod impl_render {
    impl_render_for_collections!((0, T0), (1, T1));
    impl_render_for_collections!((0, T0), (1, T1), (2, T2));
    impl_render_for_collections!((0, T0), (1, T1), (2, T2), (3, T3));
    impl_render_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4));
    impl_render_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5));
    impl_render_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6));
    impl_render_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7));
    impl_render_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8));
    impl_render_for_collections!((0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8), (9, T9));
}

impl<Color: Copy, T: Render<Color>> Render<Color> for [T] {
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = Color>, style: &Color) {
        for item in self {
            item.render(render_target, style);
        }
    }

    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
    ) {
        source
            .iter()
            .zip(target.iter())
            .for_each(|(source, target)| {
                T::render_animated(render_target, source, target, style, domain);
            });
    }

    fn render_animated_diffed(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
        differ: &mut Differ<'_>,
    ) {
        // for slices we are all or nothing, so if we aren't marked as changing,
        // then none of the children changed.
        if differ.pop() {
            Self::render_animated(render_target, source, target, style, domain);
        }
    }
}

impl<Color: Copy, T: Render<Color>, const N: usize> Render<Color> for [T; N] {
    #[inline]
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = Color>, style: &Color) {
        self.as_slice().render(render_target, style);
    }

    #[inline]
    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
    ) {
        <[T]>::render_animated(render_target, source, target, style, domain);
    }

    fn render_animated_diffed(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
        differ: &mut Differ<'_>,
    ) {
        for (a, b) in source.iter().zip(target.iter()) {
            T::render_animated_diffed(render_target, a, b, style, domain, differ);
        }
    }
}

impl<Color: Copy, T: Render<Color>, const N: usize> Render<Color> for heapless::Vec<T, N> {
    #[inline]
    fn render(&self, render_target: &mut impl RenderTarget<ColorFormat = Color>, style: &Color) {
        self.as_slice().render(render_target, style);
    }

    #[inline]
    fn render_animated(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
    ) {
        <[T]>::render_animated(render_target, source, target, style, domain);
    }

    fn render_animated_diffed(
        render_target: &mut impl RenderTarget<ColorFormat = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        domain: &AnimationDomain,
        differ: &mut Differ<'_>,
    ) {
        for (a, b) in source.iter().zip(target.iter()) {
            T::render_animated_diffed(render_target, a, b, style, domain, differ);
        }

        let remainder = N - source.len().min(target.len());

        differ.ignore(T::SIZE * remainder);
    }
}

/// Make sure tuples render in the correct order
#[cfg(test)]
mod render_order_tests {
    use crate::render::{AnimationDomain, Render};
    use std;
    use std::vec;
    use std::{cell::RefCell, rc::Rc, vec::Vec};

    #[derive(Debug, Clone, PartialEq)]
    struct OrderTracker {
        order: Rc<RefCell<Vec<usize>>>,
        id: usize,
    }

    impl OrderTracker {
        fn new(id: usize, order: Rc<RefCell<Vec<usize>>>) -> Self {
            Self { order, id }
        }
    }

    impl super::Diffable for OrderTracker {
        const SIZE: usize = 1;

        fn diff_with(&self, other: &Self, differ: &mut crate::render::Differ<'_>) {
            differ.push(self != other);
        }
    }

    impl super::AnimatedJoin for OrderTracker {
        fn join_from(&mut self, _source: &Self, _domain: &AnimationDomain) {}
    }

    impl super::IntrinsicShape for OrderTracker {
        fn content_shape(&self) -> super::ContentShape {
            super::ContentShape::Empty
        }
    }

    impl Render<char> for OrderTracker {
        fn render(
            &self,
            _render_target: &mut impl crate::render_target::RenderTarget<ColorFormat = char>,
            _style: &char,
        ) {
            self.order.borrow_mut().push(self.id);
        }

        fn render_animated(
            _render_target: &mut impl crate::render_target::RenderTarget<ColorFormat = char>,
            source: &Self,
            target: &Self,
            _style: &char,
            _domain: &AnimationDomain,
        ) {
            source.order.borrow_mut().push(source.id);
            target.order.borrow_mut().push(target.id);
        }

        fn render_animated_diffed(
            render_target: &mut impl crate::render_target::RenderTarget<ColorFormat = char>,
            source: &Self,
            target: &Self,
            style: &char,
            domain: &AnimationDomain,
            differ: &mut crate::render::Differ<'_>,
        ) {
            if differ.pop() {
                Self::render_animated(render_target, source, target, style, domain);
            }
        }
    }

    macro_rules! test_tuple_render_order {
        ($($n:tt),+) => {
            paste::paste! {
                #[test]
                fn [<tuple_render_order_$($n)*>]() {
                    let order = Rc::new(RefCell::new(Vec::new()));
                    let tuple = (
                        $(
                            OrderTracker::new($n, order.clone()),
                        )+
                    );

                    let mut mock_target = crate::render_target::FixedTextBuffer::<10, 10>::default();
                    tuple.render(&mut mock_target, &'x');

                    let expected_order: Vec<usize> = vec![$($n),+];
                    assert_eq!(*order.borrow(), expected_order);
                }

                #[test]
                fn [<tuple_animated_render_order_$($n)*>]() {
                    let order = Rc::new(RefCell::new(Vec::new()));

                    let mut mock_target = crate::render_target::FixedTextBuffer::<10, 10>::default();

                    let source_tuple = (
                        $(
                            OrderTracker::new($n, order.clone()),
                        )+
                    );
                    let target_tuple = (
                        $(
                            OrderTracker::new($n + 100, order.clone()),
                        )+
                    );

                    let domain = AnimationDomain::new(128, std::time::Duration::from_secs(1));
                    Render::render_animated(
                        &mut mock_target,
                        &source_tuple,
                        &target_tuple,
                        &'x',
                        &domain,
                    );

                    let expected_order: Vec<usize> = vec![$($n, $n + 100),+];
                    assert_eq!(*order.borrow(), expected_order);
                }
            }
        };
    }

    test_tuple_render_order!(0, 1);
    test_tuple_render_order!(0, 1, 2);
    test_tuple_render_order!(0, 1, 2, 3);
    test_tuple_render_order!(0, 1, 2, 3, 4);
    test_tuple_render_order!(0, 1, 2, 3, 4, 5);
    test_tuple_render_order!(0, 1, 2, 3, 4, 5, 6);
    test_tuple_render_order!(0, 1, 2, 3, 4, 5, 6, 7);
    test_tuple_render_order!(0, 1, 2, 3, 4, 5, 6, 7, 8);
    test_tuple_render_order!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
}

#[cfg(test)]
mod content_shape_tests {
    use super::*;
    use crate::primitives::{Point, Size};
    use crate::render::shape::Rect;

    #[test]
    fn tuple_content_shape_union() {
        let rect1 = Rect::new(Point::new(0, 0), Size::new(10, 10));
        let rect2 = Rect::new(Point::new(20, 20), Size::new(10, 10));
        let tuple = (rect1, rect2);

        let shape = tuple.content_shape();
        let bbox = shape.bounding_box().unwrap();
        assert_eq!(bbox.origin, Point::new(0, 0));
        assert_eq!(bbox.size, Size::new(30, 30));
    }

    #[test]
    fn array_content_shape_union() {
        let rects = [
            Rect::new(Point::new(0, 0), Size::new(10, 10)),
            Rect::new(Point::new(20, 20), Size::new(10, 10)),
        ];

        let shape = rects.content_shape();
        let bbox = shape.bounding_box().unwrap();
        assert_eq!(bbox.origin, Point::new(0, 0));
        assert_eq!(bbox.size, Size::new(30, 30));
    }

    #[test]
    fn heapless_vec_content_shape_union() {
        let mut vec: heapless::Vec<Rect, 4> = heapless::Vec::new();
        vec.push(Rect::new(Point::new(0, 0), Size::new(10, 10)))
            .ok();
        vec.push(Rect::new(Point::new(20, 20), Size::new(10, 10)))
            .ok();

        let shape = vec.content_shape();
        let bbox = shape.bounding_box().unwrap();
        assert_eq!(bbox.origin, Point::new(0, 0));
        assert_eq!(bbox.size, Size::new(30, 30));
    }

    #[test]
    fn empty_array_content_shape_is_empty() {
        let rects: [Rect; 0] = [];
        assert_eq!(rects.content_shape(), ContentShape::Empty);
    }

    #[test]
    fn tuple_union_of_bounding_boxes() {
        let rect = Rect::new(Point::new(20, 20), Size::new(10, 10));
        let rects = (Option::<Rect>::None, rect.clone());
        assert_eq!(
            rects.content_shape().bounding_box(),
            rect.content_shape().bounding_box()
        );
    }
}
