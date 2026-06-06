use core::{cell::Cell, time::Duration};

use crate::{
    focus::{self, FocusAction, FocusGroup, RoleSet},
    primitives::Point,
    render::ContentShape,
};

/// An interaction event that can be handled by a view.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A request to move focus, often driven by navigational buttons
    /// or an encoder.
    Focus {
        /// The focus action to perform.
        action: FocusAction,
        /// The focus group this event targets.
        group: FocusGroup,
    },
    /// A key was pressed.
    KeyDown(u8),
    /// A key was released.
    KeyUp(u8),
}

impl From<FocusAction> for Event {
    fn from(action: FocusAction) -> Self {
        Self::Focus {
            action,
            group: focus::GROUP_0,
        }
    }
}

impl Event {
    /// Returns a new event with the specified offset applied to any point-based data.
    #[must_use]
    pub fn offset(&self, _offset: Point) -> Self {
        self.clone()
    }

    /// Returns a new event with the specified focus group set on Focus events.
    ///
    /// Non-focus events are returned unchanged.
    #[must_use]
    pub fn with_focus_group(mut self, group: FocusGroup) -> Self {
        if let Self::Focus {
            group: ref mut g, ..
        } = self
        {
            *g = group;
        }
        self
    }
}

/// The result of handling an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    /// Focus successfully moved to a new element
    Handled {
        /// The content shape of the focused element
        shape: ContentShape,
        /// The element which handled the event has focus
        request_focus: bool,
        /// The group of the focused element
        group: FocusGroup,
    },
    /// The event was not handled, or focus not obtained
    Deferred { focus_lost: bool },
}

impl Default for EventResult {
    fn default() -> Self {
        Self::Deferred { focus_lost: false }
    }
}

impl EventResult {
    /// Creates a new `EventResult` indicating the event was handled but focus was not obtained.
    #[must_use]
    pub const fn handled_unfocused() -> Self {
        Self::Handled {
            shape: ContentShape::Empty,
            request_focus: false,
            group: focus::GROUP_0,
        }
    }

    /// Creates a new `EventResult` indicating the event was handled with focus.
    #[must_use]
    pub const fn handled_focused(shape: ContentShape) -> Self {
        Self::Handled {
            shape,
            request_focus: true,
            group: focus::GROUP_0,
        }
    }

    /// Creates a new `EventResult` indicating the event was deferred.
    #[must_use]
    pub const fn deferred() -> Self {
        Self::Deferred { focus_lost: false }
    }

    /// Creates a new `EventResult` indicating the event was deferred and focus was lost.
    #[must_use]
    pub const fn deferred_lost_focus() -> Self {
        Self::Deferred { focus_lost: true }
    }

    /// Returns a new [`EventResult`] with the specified focus group.
    #[must_use]
    pub const fn with_group(mut self, group: FocusGroup) -> Self {
        let Self::Handled {
            group: event_group, ..
        } = &mut self
        else {
            return Self::deferred();
        };
        *event_group = group;
        self
    }

    /// Returns true if the event was handled (not deferred).
    #[must_use]
    pub const fn is_handled(&self) -> bool {
        matches!(self, Self::Handled { .. })
    }

    /// Returns true if this result is from an element requesting focus.
    #[must_use]
    pub const fn requested_focus(&self) -> bool {
        matches!(
            self,
            Self::Handled {
                request_focus: true,
                ..
            }
        )
    }

    /// Returns true if this result is from an element requesting focus.
    #[must_use]
    pub const fn lost_focus(&self) -> bool {
        matches!(
            self,
            Self::Deferred {
                focus_lost: true,
                ..
            }
        )
    }

    /// Returns the content shape if this result has one.
    #[must_use]
    pub fn shape(&self) -> Option<&ContentShape> {
        match self {
            Self::Handled { shape, .. } => Some(shape),
            Self::Deferred { .. } => None,
        }
    }

    /// Returns the result offset by the given amount if it has a shape.
    #[must_use]
    pub fn with_offset(mut self, offset: Point) -> Self {
        match &mut self {
            Self::Handled { shape, .. } => {
                shape.offset(offset);
            }
            Self::Deferred { .. } => (),
        }
        self
    }
}

/// Context provided to views when handling events.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventContext {
    /// The time since the application started.
    pub app_time: Duration,
    /// Whether a redraw has been requested.
    pub redraw_requested: Cell<bool>,
    /// Whether a view rebuild has been requested.
    pub view_rebuild_requested: Cell<bool>,
    /// Which roles are currently active.
    pub roles: RoleSet,
}

impl EventContext {
    /// Creates a new `EventContext` with the given application time.
    #[must_use]
    pub const fn new(app_time: Duration) -> Self {
        Self {
            app_time,
            redraw_requested: Cell::new(false),
            view_rebuild_requested: Cell::new(false),
            roles: RoleSet::any(),
        }
    }

    #[must_use]
    pub fn with_roles(mut self, roles: impl Into<RoleSet>) -> Self {
        self.roles = roles.into();
        self
    }

    /// Indicates to the render loop that the view needs to be rebuilt.
    pub fn request_view_rebuild(&self) {
        self.view_rebuild_requested.set(true);
    }

    /// This flag indicates the view should be redrawn even if no animations were reported as
    /// active.
    ///
    /// This should be set when a view directly modifies the render tree state
    /// without requesting a view recompute, e.g. scrollview dragging.
    pub fn request_redraw(&self) {
        self.redraw_requested.set(true);
    }
}
