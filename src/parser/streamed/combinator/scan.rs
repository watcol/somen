use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`scan`].
///
/// [`scan`]: crate::parser::streamed::StreamedParserExt::scan
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scan<P, Q, F> {
    inner: P,
    init: Q,
    f: F,
}

impl<P, Q, F> Scan<P, Q, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, init: Q, f: F) -> Self {
        Self { inner, init, f }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.init)
    }
}

crate::parser_state! {
    pub struct ScanState<I, P: StreamedParser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt(get_mut = get_acc)]
        acc: Q::Output,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, F, T, I> StreamedParser<I> for Scan<P, Q, F>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    F: FnMut(&mut Q::Output, P::Item) -> Option<T>,
    I: Positioned + ?Sized,
{
    type Item = T;
    type State = ScanState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.init.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(acc, err) => {
                    state.error = err;
                    state.inner = EitherState::new_right();
                    state.acc = Some(acc);
                }
                Status::Failure(err, exclusive) => {
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)))
                }
            }
        }

        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.right())?)
            {
                Status::Success(Some(i), err) => match (self.f)(state.get_acc(), i) {
                    Some(val) => {
                        merge_errors(&mut state.error, err);
                        break Status::Success(Some(val), state.error());
                    }
                    None => {
                        merge_errors(&mut state.error, err);
                    }
                },
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(None, state.error());
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => break Status::Failure(err, true),
            }
        }))
    }
}

/// A parser for method [`try_scan`].
///
/// [`try_scan`]: crate::parser::streamed::StreamedParserExt::try_scan
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryScan<P, Q, F> {
    inner: P,
    init: Q,
    f: F,
}

impl<P, Q, F> TryScan<P, Q, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, init: Q, f: F) -> Self {
        Self { inner, init, f }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.init)
    }
}

crate::parser_state! {
    pub struct TryScanState<I, P: StreamedParser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt(get_mut = get_acc)]
        acc: Q::Output,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, F, T, E, I> StreamedParser<I> for TryScan<P, Q, F>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    F: FnMut(&mut Q::Output, P::Item) -> Result<Option<T>, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Item = T;
    type State = TryScanState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.init.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(acc, err) => {
                    state.error = err;
                    state.inner = EitherState::new_right();
                    state.acc = Some(acc);
                }
                Status::Failure(err, exclusive) => {
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)))
                }
            }
        }

        Poll::Ready(Ok(loop {
            state.set_start(|| input.position());
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.right())?)
            {
                Status::Success(Some(i), err) => match (self.f)(state.get_acc(), i) {
                    Ok(Some(val)) => {
                        merge_errors(&mut state.error, err);
                        state.start = None;
                        break Status::Success(Some(val), state.error());
                    }
                    Ok(None) => {
                        merge_errors(&mut state.error, err);
                        state.start = None;
                    }
                    Err(exp) => {
                        break Status::Failure(
                            Error {
                                expects: exp.into(),
                                position: state.start()..input.position(),
                            },
                            true,
                        );
                    }
                },
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    state.start = None;
                    break Status::Success(None, state.error());
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    state.start = None;
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => {
                    state.start = None;
                    break Status::Failure(err, true);
                }
            }
        }))
    }
}
