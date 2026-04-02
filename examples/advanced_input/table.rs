use crate::FONT;
use crate::definitions::{GoodPixelColor, IeName, MAX_COLS, MAX_ROWS, Palette, RenderData, State};
use buoyant::view::prelude::*;
use heapless::String;

use std::fmt::Write;

mod ie_editor;

fn funnel<F: Fn(&mut State)>(f: F) -> F {
    f
}

pub fn table<'a, C: GoodPixelColor>(
    data: RenderData<'a, C>,
    state: &'_ State,
    table_dimensions: (usize, usize),
    names: &'a [Option<IeName<'a>>],
    ie: &'a [Option<f32>],
    eu: &'a [Option<&'static str>],
) -> impl View<C, State> + use<'a, C> {
    const fn index_array<const N: usize>() -> [usize; N] {
        let mut arr = [0; N];
        let mut i = 0;
        while i < N {
            arr[i] = i;
            i += 1;
        }
        arr
    }

    const WIDTH: [usize; MAX_COLS] = index_array();
    const HEIGHT: [usize; MAX_ROWS] = index_array();

    let (c, r) = table_dimensions;

    let enter_ie = move |index: u8| {
        let ie = ie[index as usize];
        funnel(move |s: &mut State| {
            if let Some(ie) = ie {
                s.opened_cell_input = Some(index);
                s.temporary_ie = ie.into();
            }
        })
    };
    let set_ie = move |index: u8| {
        funnel(move |s| {
            s.ie_value_update = Some((index, s.temporary_ie.into()));
            s.opened_cell_input = None;
        })
    };
    let cancel_ie = funnel(|s: &mut State| s.opened_cell_input = None);

    let overlay = state.opened_cell_input.map(|i| (i, state.temporary_ie));

    // Not a table in this example
    ForEach::<MAX_COLS>::new_horizontal(&WIDTH[..c], move |i| {
        ForEach::<MAX_ROWS>::new_vertical(&HEIGHT[..r], move |j| {
            let index = j * c + i;
            Button::new(enter_ie(index as u8), move |s| {
                let (name, ie, eu) = (names[index], ie[index], eu[index]);
                let white = data.palette.white();
                let background = if s.is_focused() | s.is_pressed() {
                    Rectangle.stroked(4).foreground_color(white)
                } else {
                    Rectangle.stroked(0).foreground_color(white)
                };
                buoyant::match_view!((name, ie), {
                    (Some(name), Some(ie)) => ie_cell(name, ie, eu, data.palette),
                    (None, Some(ie)) => ie_cell(IeName::Known(""), ie, eu, data.palette),
                    (_, None) => EmptyView,
                })
                .background(Alignment::Center, Rectangle.stroked(1))
                .frame_sized(321 / c as u32, 204 / r as u32)
                .background(Alignment::Center, background)
            })
        })
    })
    .background(Alignment::Center, Rectangle.stroked(2))
    .popover(overlay.as_ref(), |(i, ie)| {
        ie_editor::ie_editor(*ie, set_ie(*i), cancel_ie, data.palette)
    })
}

fn any_view<C: GoodPixelColor, State>() -> impl View<C, State> {
    Rectangle
        .popover(Some(&0), |_| any_view_2())
}

fn any_view_2<C: GoodPixelColor, State>() -> impl View<C, State> {
    Rectangle
}

fn any_render<C: GoodPixelColor>() -> impl buoyant::render::Render<C> {
    let a = buoyant::render::Rect {
        origin: buoyant::primitives::Point::new(0, 0),
        size: buoyant::primitives::Size::new(0, 0),
    };

    let subtree = buoyant::render::TransitionOption::Some {
            subtree: a.clone(),
            size: a.size.clone(),
            transition: buoyant::transition::Opacity,
        };
    let b = buoyant::render::Animate::new(
        subtree,
        buoyant::animation::Animation::linear(core::time::Duration::from_secs(1)),
        core::time::Duration::from_secs(1),
        true,
    );

    let r = (a, b);
    r
}

// fn foo<T: View<C, State>, C, State>(t: T) {}
// fn bar<C: Copy>() {
//     // let a = Capsule;
//     let a = ForEach::<10>::new_horizontal(&[1, 2], move |i| {
//     });
//     foo::<_, C, ()>(a);
// }

fn ie_cell<C: GoodPixelColor>(
    name: IeName<'_>,
    ie: f32,
    eu: Option<&'static str>,
    palette: &'static Palette<C>,
) -> impl View<C, State> {
    let mut name_buf = String::<16, u8>::new();
    let mut value_buf = String::<8, u8>::new();
    let eu = eu.unwrap_or("");

    // --- name ---
    let overflow = match name {
        IeName::Known(name) => name_buf.push_str(&name[..name.len().min(16)]).is_err(),
        IeName::Addr((c1, c2), (i1, i2, i3)) => {
            write!(name_buf, "{c1}.{c2} {i1}.{i2}.{i3}").is_err()
        }
    };

    // --- value ---
    let value = ((ie * 100.0) / 100.0).trunc();
    let overflow = write!(&mut value_buf, "{value}").is_err() || overflow;

    assert!(!overflow, "Overflow of cell");

    // --- quality ---
    // let vcolor = value_color(ie.try_get_qds(), overflow, palette);

    HStack::new((
        Text::new(name_buf, &FONT),
        Spacer::default(),
        Text::new(value_buf, &FONT).foreground_color(palette.green()),
        Text::new(eu, &FONT),
    ))
    .with_spacing(4)
    .padding(Edges::Vertical, 2)
    .padding(Edges::Horizontal, 6)
    .flex_infinite_width(HorizontalAlignment::Center)
    .flex_infinite_height(VerticalAlignment::Center)
}
