use crate::{
    environment::LayoutEnvironment,
    event::{EventContext, EventResult},
    layout::ResolvedLayout,
    primitives::{Point, ProposedDimensions},
    view::{ViewLayout, ViewMarker},
};

/// Converts the captures of a parent view to a subview's captures.
#[derive(Debug, Clone)]
pub struct Lens<InnerView, CaptureFn> {
    inner: InnerView,
    capture_fn: CaptureFn,
}

impl<InnerView: ViewMarker, CaptureFn> Lens<InnerView, CaptureFn> {
    #[allow(missing_docs)]
    #[must_use]
    pub fn new<OuterCapture, InnerCapture>(inner: InnerView, capture_fn: CaptureFn) -> Self
    where
        InnerView: ViewLayout<InnerCapture>,
        CaptureFn: Fn(&mut OuterCapture) -> &mut InnerCapture,
    {
        Self { inner, capture_fn }
    }
}

impl<InnerView, F> ViewMarker for Lens<InnerView, F>
where
    InnerView: ViewMarker,
{
    type Renderables = InnerView::Renderables;
    type Transition = InnerView::Transition;
}

impl<
    InnerView: ViewLayout<InnerCaptures>,
    Captures: ?Sized,
    F: for<'a> Fn(&'a mut Captures) -> &'a mut InnerCaptures,
    InnerCaptures,
> ViewLayout<Captures> for Lens<InnerView, F>
{
    type State = InnerView::State;
    type Sublayout = InnerView::Sublayout;
    type FocusTree = InnerView::FocusTree;

    fn priority(&self) -> i8 {
        self.inner.priority()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn transition(&self) -> Self::Transition {
        self.inner.transition()
    }

    fn build_state(&self, captures: &mut Captures) -> Self::State {
        let inner_captures = (self.capture_fn)(captures);
        self.inner.build_state(inner_captures)
    }

    #[inline(never)]
    fn layout(
        &self,
        offer: &ProposedDimensions,
        env: &impl LayoutEnvironment,
        captures: &mut Captures,
        state: &mut Self::State,
    ) -> ResolvedLayout<Self::Sublayout> {
        self.inner
            .layout(offer, env, (self.capture_fn)(captures), state)
    }

    #[inline(never)]
    fn render_tree(
        &self,
        layout: &Self::Sublayout,
        origin: Point,
        env: &impl LayoutEnvironment,
        captures: &mut Captures,
        state: &mut Self::State,
    ) -> Self::Renderables {
        self.inner
            .render_tree(layout, origin, env, (self.capture_fn)(captures), state)
    }

    #[inline(never)]
    fn handle_event(
        &self,
        event: &super::Event,
        context: &EventContext,
        render_tree: &mut Self::Renderables,
        captures: &mut Captures,
        state: &mut Self::State,
        focus: &mut Self::FocusTree,
    ) -> EventResult {
        self.inner.handle_event(
            event,
            context,
            render_tree,
            (self.capture_fn)(captures),
            state,
            focus,
        )
    }
}

/// Converts the captures of a parent view to a subview's captures.
#[derive(Debug, Clone)]
pub struct StatefulLens<S, InnerView, CaptureFn, InitFn> {
    _data: core::marker::PhantomData<S>,
    inner: InnerView,
    capture_fn: CaptureFn,
    init_fn: InitFn,
}

impl<S, InnerView: ViewMarker, CaptureFn, InitFn> StatefulLens<S, InnerView, CaptureFn, InitFn> {
    #[allow(missing_docs)]
    #[must_use]
    pub fn new<OuterCapture, InnerCapture>(
        inner: InnerView,
        capture_fn: CaptureFn,
        init_fn: InitFn,
    ) -> Self
    where
        InnerView: ViewLayout<InnerCapture>,
        CaptureFn: for<'a> Fn(&'a mut S, &'a mut OuterCapture) -> &'a mut InnerCapture,
        InitFn: Fn(&mut OuterCapture) -> (S, InnerCapture),
    {
        Self {
            _data: core::marker::PhantomData,
            inner,
            capture_fn,
            init_fn,
        }
    }
}

impl<S, InnerView, F, IF> ViewMarker for StatefulLens<S, InnerView, F, IF>
where
    InnerView: ViewMarker,
{
    type Renderables = InnerView::Renderables;
    type Transition = InnerView::Transition;
}

impl<
    S: 'static,
    InnerView: ViewLayout<InnerCaptures>,
    Captures: ?Sized,
    F: for<'a> Fn(&'a mut S, &'a mut Captures) -> &'a mut InnerCaptures,
    IF: Fn(&mut Captures) -> (S, InnerCaptures),
    InnerCaptures,
> ViewLayout<Captures> for StatefulLens<S, InnerView, F, IF>
{
    type State = (S, InnerView::State);
    type Sublayout = InnerView::Sublayout;
    type FocusTree = InnerView::FocusTree;

    fn priority(&self) -> i8 {
        self.inner.priority()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn transition(&self) -> Self::Transition {
        self.inner.transition()
    }

    fn build_state(&self, captures: &mut Captures) -> Self::State {
        let (s, mut inner_captures): (S, InnerCaptures) = (self.init_fn)(captures);
        (s, self.inner.build_state(&mut inner_captures))
    }

    #[inline(never)]
    fn layout(
        &self,
        offer: &ProposedDimensions,
        env: &impl LayoutEnvironment,
        captures: &mut Captures,
        state: &mut Self::State,
    ) -> ResolvedLayout<Self::Sublayout> {
        self.inner.layout(
            offer,
            env,
            (self.capture_fn)(&mut state.0, captures),
            &mut state.1,
        )
    }

    #[inline(never)]
    fn render_tree(
        &self,
        layout: &Self::Sublayout,
        origin: Point,
        env: &impl LayoutEnvironment,
        captures: &mut Captures,
        state: &mut Self::State,
    ) -> Self::Renderables {
        self.inner.render_tree(
            layout,
            origin,
            env,
            (self.capture_fn)(&mut state.0, captures),
            &mut state.1,
        )
    }

    #[inline(never)]
    fn handle_event(
        &self,
        event: &super::Event,
        context: &EventContext,
        render_tree: &mut Self::Renderables,
        captures: &mut Captures,
        state: &mut Self::State,
        focus: &mut Self::FocusTree,
    ) -> EventResult {
        self.inner.handle_event(
            event,
            context,
            render_tree,
            (self.capture_fn)(&mut state.0, captures),
            &mut state.1,
            focus,
        )
    }
}
