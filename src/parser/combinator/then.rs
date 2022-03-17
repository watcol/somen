use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`then`].
///
/// [`then`]: crate::parser::ParserExt::then
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Then<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Then<P, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct ThenState<I, P: Parser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        parser: Q,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, F, Q, I> Parser<I> for Then<P, F>
where
    P: Parser<I>,
    F: FnMut(P::Output) -> Q,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = ThenState<I, P, Q>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.inner.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(val, err) => {
                    state.parser = Some((self.f)(val));
                    state.inner = EitherState::new_right();
                    state.error = err;
                }
                Status::Failure(err, exclusive) => {
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)))
                }
            }
        }

        state
            .parser
            .as_mut()
            .unwrap()
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                Status::Success(val, err) => {
                    merge_errors(&mut state.error, err);
                    Status::Success(val, state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                exclusive => exclusive,
            })
    }
}

crate::parser_state! {
    pub struct ThenStreamedState<I, P: Parser, Q: StreamedParser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        parser: Q,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, F, Q, I> StreamedParser<I> for Then<P, F>
where
    P: Parser<I>,
    F: FnMut(P::Output) -> Q,
    Q: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Item = Q::Item;
    type State = ThenStreamedState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.inner.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(val, err) => {
                    state.parser = Some((self.f)(val));
                    state.inner = EitherState::new_right();
                    state.error = err;
                }
                Status::Failure(err, exclusive) => {
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)))
                }
            }
        }

        state
            .parser
            .as_mut()
            .unwrap()
            .poll_parse_next(input, cx, state.inner.right())
            .map_ok(|status| match status {
                Status::Success(val, err) => {
                    merge_errors(&mut state.error, err);
                    Status::Success(val, state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                exclusive => exclusive,
            })
    }
}

/// A parser for method [`try_then`].
///
/// [`try_then`]: crate::parser::ParserExt::try_then
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryThen<P, F> {
    inner: P,
    f: F,
}

impl<P, F> TryThen<P, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct TryThenState<I, P: Parser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        parser: Q,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, F, Q, E, I> Parser<I> for TryThen<P, F>
where
    P: Parser<I>,
    F: FnMut(P::Output) -> Result<Q, E>,
    Q: Parser<I>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = TryThenState<I, P, Q>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            if state.start.is_none() {
                state.start = Some(input.position());
            }

            match ready!(self.inner.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(val, err) => match (self.f)(val) {
                    Ok(parser) => {
                        state.parser = Some(parser);
                        state.inner = EitherState::new_right();
                        state.error = err;
                    }
                    Err(exp) => {
                        return Poll::Ready(Ok(Status::Failure(
                            Error {
                                expects: exp.into(),
                                position: state.start()..input.position(),
                            },
                            true,
                        )))
                    }
                },
                Status::Failure(err, exclusive) => {
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)));
                }
            }
        }

        state
            .parser
            .as_mut()
            .unwrap()
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                Status::Success(val, err) => {
                    merge_errors(&mut state.error, err);
                    Status::Success(val, state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                exclusive => exclusive,
            })
    }
}

crate::parser_state! {
    pub struct TryThenStreamedState<I, P: Parser, Q: StreamedParser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        parser: Q,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, F, Q, E, I> StreamedParser<I> for TryThen<P, F>
where
    P: Parser<I>,
    F: FnMut(P::Output) -> Result<Q, E>,
    Q: StreamedParser<I>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Item = Q::Item;
    type State = TryThenStreamedState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            if state.start.is_none() {
                state.start = Some(input.position());
            }

            match ready!(self.inner.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(val, err) => match (self.f)(val) {
                    Ok(parser) => {
                        state.parser = Some(parser);
                        state.inner = EitherState::new_right();
                        state.error = err;
                    }
                    Err(exp) => {
                        return Poll::Ready(Ok(Status::Failure(
                            Error {
                                expects: exp.into(),
                                position: state.start()..input.position(),
                            },
                            true,
                        )));
                    }
                },
                Status::Failure(err, exclusive) => {
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)))
                }
            }
        }

        state
            .parser
            .as_mut()
            .unwrap()
            .poll_parse_next(input, cx, state.inner.right())
            .map_ok(|status| match status {
                Status::Success(val, err) => {
                    merge_errors(&mut state.error, err);
                    Status::Success(val, state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                exclusive => exclusive,
            })
    }
}
