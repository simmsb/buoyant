use crate::{
    event::{Event, EventResult},
    view::{ViewLayout, ViewMarker},
};

#[derive(Debug)]
pub struct CapturesEvent<V, F> {
    inner: V,
    mapping: F,
}

impl<V, F> CapturesEvent<V, F> {
    #[must_use]
    pub const fn new(inner: V, mapping: F) -> Self {
        Self { inner, mapping }
    }
}

impl<V: ViewMarker, F> ViewMarker for CapturesEvent<V, F> {
    type Renderables = V::Renderables;
    type Transition = V::Transition;
}

impl<C: ?Sized, V: ViewLayout<C>, F: Fn(&Event, &mut C) -> Option<Event>> ViewLayout<C>
    for CapturesEvent<V, F>
{
    type State = V::State;

    type Sublayout = V::Sublayout;

    type FocusTree = V::FocusTree;

    fn priority(&self) -> i8 {
        self.inner.priority()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn transition(&self) -> Self::Transition {
        self.inner.transition()
    }

    fn build_state(&self, captures: &mut C) -> Self::State {
        self.inner.build_state(captures)
    }

    fn layout(
        &self,
        offer: &crate::primitives::ProposedDimensions,
        env: &impl crate::environment::LayoutEnvironment,
        captures: &mut C,
        state: &mut Self::State,
    ) -> crate::layout::ResolvedLayout<Self::Sublayout> {
        self.inner.layout(offer, env, captures, state)
    }

    fn render_tree(
        &self,
        layout: &Self::Sublayout,
        origin: crate::primitives::Point,
        env: &impl crate::environment::LayoutEnvironment,
        captures: &mut C,
        state: &mut Self::State,
    ) -> Self::Renderables {
        self.inner.render_tree(layout, origin, env, captures, state)
    }

    fn handle_event(
        &self,
        event: &Event,
        context: &crate::event::EventContext,
        render_tree: &mut Self::Renderables,
        captures: &mut C,
        state: &mut Self::State,
        focus: &mut Self::FocusTree,
    ) -> crate::event::EventResult {
        let mapped_event = (self.mapping)(event, captures);
        if let Some(mapped_event) = mapped_event {
            self.inner
                .handle_event(&mapped_event, context, render_tree, captures, state, focus)
        } else {
            EventResult::deferred()
        }
    }
}
