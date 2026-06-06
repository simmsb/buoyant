use buoyant::{
    event::Event,
    focus::{self, FocusAction},
    view::{View, prelude::*, scroll_view::ScrollBarVisibility},
};
use strum::{EnumCount as _, VariantArray};
use std::fmt::Write as _;

use crate::{
    ui::{
        colour::{self, ColorFormat},
        font, keys,
        state::{self},
    },
};

#[must_use]
pub fn view(_state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    ScrollView::new(
        ForEach::<{ Info::COUNT }>::new_vertical(Info::VARIANTS, move |s| info_entry(*s))
            .with_spacing(8),
    )
    .with_bar_visibility(ScrollBarVisibility::Never)
    .padding(Edges::All, 8)
    .captures_event(|e, s: &mut state::State| match e {
        Event::KeyDown(keys::UP_CLICK) => Some(FocusAction::Previous.into_event(focus::GROUP_0)),
        Event::KeyDown(keys::DOWN_CLICK) => Some(FocusAction::Next.into_event(focus::GROUP_0)),
        Event::KeyDown(keys::CONFIRM_HOLD) => {
            s.page_action = Some(state::PageAction::ExitSettings);
            None
        }
        _ => Some(e.clone()),
    })
}

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, strum::EnumCount, strum::VariantArray)]
pub enum Info {
    SystemVoltageController,
    SystemVoltageBattery,
    BatteryCurrent,
    BatteryCommand,
    BatteryState,
    BatteryRange,
    BatteryRelSOC,
    BatteryAbsSOC,
    BatteryRelSOH,
    BatteryAbsSOH,
    BatteryCapacity,
    BatteryCharging,
    BatteryCharged,
    BatteryTemperature,
    GitCommit,
    Dummy,
}

impl Info {
    fn name(self) -> &'static str {
        match self {
            Info::SystemVoltageController => "Voltage (ctrl)",
            Info::SystemVoltageBattery => "Voltage (bat)",
            Info::BatteryCurrent => "Current",
            Info::BatteryCommand => "Bat Cmd",
            Info::BatteryState => "Bat State",
            Info::BatteryRange => "Bat Range",
            Info::BatteryRelSOC => "Rel SoC",
            Info::BatteryAbsSOC => "Abs SoC",
            Info::BatteryRelSOH => "Rel SoH",
            Info::BatteryAbsSOH => "Abs SoH",
            Info::BatteryCapacity => "Capacity",
            Info::BatteryCharging => "Charging",
            Info::BatteryCharged => "Charged",
            Info::BatteryTemperature => "Bat temp",
            Info::GitCommit => "Git commit",
            Info::Dummy => "",
        }
    }

    fn val(self) -> heapless::String<8, u8> {
        let mut s = heapless::String::new();

        match self {
            Info::SystemVoltageController => {
                let _ = write!(&mut s, "0");
            }
            Info::SystemVoltageBattery => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryCurrent => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryCommand => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryState => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryRange => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryRelSOC => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryAbsSOC => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryRelSOH => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryAbsSOH => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryCapacity => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryCharging => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryCharged => {
                let _ = write!(&mut s, "0");
            }
            Info::BatteryTemperature => {
                let _ = write!(&mut s, "0");
            }
            Info::GitCommit => {
                let _ = s.write_str("aaa");
            }
            Info::Dummy => {}
        };

        s
    }
}

fn info_entry(info: Info) -> impl View<ColorFormat, state::State> + use<> {
    Button::new(
        move |_s: &mut state::State| {},
        move |bs| {
            let (fg, bg) = if bs.is_focused() {
                (colour::ON_TERTIARY_CONTAINER, colour::TERTIARY_CONTAINER)
            } else {
                (colour::ON_SECONDARY_CONTAINER, colour::SECONDARY_CONTAINER)
            };

            HStack::new((
                Text::new(info.name(), &font::B612_REGULAR).foreground_color(fg),
                Text::new(info.val(), &font::B612_SMALL)
                    .foreground_color(fg)
                    .flex_infinite_width(HorizontalAlignment::Trailing),
            ))
            .with_alignment(VerticalAlignment::Center)
            .with_spacing(4)
            .padding(Edges::All, 8)
            .flex_infinite_width(HorizontalAlignment::Leading)
            .background_color(bg, RoundedRectangle::new(8))
            .padding(Edges::Horizontal, 8)
        },
    )
}
