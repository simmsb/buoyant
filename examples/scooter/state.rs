#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum Page {
    Locked,
    #[default]
    Home,
    Settings,
}

impl Page {
    pub fn handle_action(&self, action: PageAction) -> Option<Self> {
        match (self, action) {
            (Page::Locked, PageAction::Unlock) => Some(Page::Home),
            (Page::Locked, _) => return None,
            (Page::Home, PageAction::Lock) => Some(Page::Locked),
            (Page::Home, PageAction::EnterSettings) => Some(Page::Settings),
            (Page::Home, _) => return None,
            (Page::Settings, PageAction::Lock) => Some(Page::Locked),
            (Page::Settings, PageAction::ExitSettings) => Some(Page::Home),
            (Page::Settings, _) => return None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PageAction {
    Lock,
    Unlock,
    EnterSettings,
    ExitSettings,
}

#[derive(PartialEq, Eq, Clone, Copy, rotate_enum::RotateEnum)]
pub enum SpeedMode {
    Sport,
    Drive,
    Eco,
    Walk,
}

impl SpeedMode {
    pub fn increase(self) -> Self {
        if self == SpeedMode::Sport {
            self
        } else {
            self.prev()
        }
    }

    pub fn decrease(self) -> Self {
        if self == SpeedMode::Walk {
            self
        } else {
            self.next()
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SpeedMode::Sport => "Sport",
            SpeedMode::Drive => "Drive",
            SpeedMode::Eco => "Eco",
            SpeedMode::Walk => "Walk",
        }
    }
}

pub struct State {
    pub page: Page,

    pub locked_state: super::view::locked::State,

    pub system_state: SystemState,
    pub operation_state: OperationState,

    pub page_action: Option<PageAction>,
    pub next_speed_mode: Option<SpeedMode>,
}

impl State {
    pub fn new() -> Self {
        Self {
            page: Default::default(),
            locked_state: Default::default(),
            system_state: SystemState::DEFAULT,
            operation_state: OperationState::Active(ActiveState {
                throttle: Throttle(123),
                speed_limit: 25,
                speed_mode: SpeedMode::Eco,
                headlight_mode: HeadlightMode::Auto {
                    low: AmbientLight(10),
                    high: AmbientLight(20),
                    currently_on: false,
                },
            }),
            page_action: None,
            next_speed_mode: None,
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct BatteryLevel {
    pub from_controller: u8,
    pub from_battery: u8,
}

#[derive(PartialEq, Eq, Clone)]
pub struct SystemVoltage {
    pub(crate) from_controller: u16,
    pub(crate) from_battery: u32,
}

#[derive(PartialEq, Eq, Clone)]
pub struct BatteryDebug {
    pub(crate) command: u16,
    pub(crate) state: u16,
    pub(crate) estimated_range: u32,
}

#[derive(PartialEq, Eq, Clone)]
pub struct BatteryInfo {
    pub(crate) relative_soc: u32,
    pub(crate) absolute_soc: u32,
    pub(crate) relative_soh: u8,
    pub(crate) absolute_soh: u32,
    pub(crate) capacity: u16,
    pub(crate) charged: bool,
    pub(crate) temperature: i16,
}

#[derive(PartialEq, Eq, Default, Clone)]
pub struct BatteryChargeEntry {
    pub(crate) when: (),
    pub(crate) charge: u16,
}

#[derive(PartialEq, Eq, Clone)]
pub struct SystemState {
    pub motor_speed: u16,
    pub headlight_on: bool,
    pub brake_light_on: bool,

    pub controller_temp: u8,
    pub system_voltage: SystemVoltage,
    pub controller_speed_limit_mode: bool,

    pub battery_current: i32,
    pub battery_level: BatteryLevel,
    pub battery_debug: BatteryDebug,
    pub battery_info: BatteryInfo,

    pub throttle: Throttle,
    pub ambient_light: AmbientLight,

    pub buttons: Buttons,

    pub charges: [Option<BatteryChargeEntry>; 16],
}

impl SystemState {
    const DEFAULT: Self = SystemState {
        motor_speed: 0,
        headlight_on: false,
        brake_light_on: false,
        controller_temp: 0,
        system_voltage: SystemVoltage {
            from_controller: 52444,
            from_battery: 54300,
        },
        controller_speed_limit_mode: false,
        battery_current: 12345,
        battery_level: BatteryLevel {
            from_controller: 100,
            from_battery: 100,
        },
        battery_debug: BatteryDebug {
            command: 0,
            state: 0,
            estimated_range: 0,
        },
        battery_info: BatteryInfo {
            relative_soc: 0,
            absolute_soc: 0,
            relative_soh: 0,
            absolute_soh: 0,
            capacity: 0,
            charged: false,
            temperature: 0,
        },
        throttle: Throttle::INITIAL,
        ambient_light: AmbientLight::INITIAL,
        buttons: Buttons(0),

        charges: [const { None }; _],
    };
}

#[derive(PartialEq, Eq, Clone)]
pub enum HeadlightMode {
    Auto {
        /// Headlight will switch on when ambient light reads under this
        low: AmbientLight,

        /// Headlight will switch off when ambient light reads over this
        high: AmbientLight,

        currently_on: bool,
    },
    On,
    Off,
}

#[derive(PartialEq, Eq, Clone)]
pub struct ActiveState {
    pub throttle: Throttle,

    /// Speed limit in km/h, we'll later use this to select the 25/35/45 limit
    /// sent to the controller
    pub speed_limit: u8,

    pub speed_mode: SpeedMode,

    pub headlight_mode: HeadlightMode,
}

#[derive(Eq, PartialEq, Default, Clone, Copy)]
pub struct Throttle(pub u16);

impl Throttle {
    pub const INITIAL: Self = Self(0);
}

#[derive(Eq, PartialEq, Default, Clone, Copy)]
pub struct AmbientLight(pub u8);

impl AmbientLight {
    pub const INITIAL: Self = Self(0);
}

#[derive(PartialEq, Eq, Clone)]
pub struct Buttons(u8);

#[derive(PartialEq, Eq, Clone)]
pub enum OperationState {
    Locked,
    Active(ActiveState),
}

impl OperationState {
    pub fn as_active(&self) -> Option<&ActiveState> {
        if let OperationState::Active(active) = self {
            Some(active)
        } else {
            None
        }
    }
}

impl OperationState {
    const DEFAULT: Self = Self::Locked;
}
