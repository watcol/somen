use core::ops::Range;
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
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, init: Q, f: F) -> Self {
        Self { inner, init, f }
    }

    /// Extracting the inner parser.
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
        #[opt(set = set_start)]
        start: I::Locator,
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
                (Status::Success(acc, err), pos) => {
                    state.start = Some(pos.start);
                    state.error = err;
                    state.inner = EitherState::new_right();
                    state.acc = Some(acc);
                }
                (Status::Failure(err, exclusive), pos) => {
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)))
                }
            }
        }

        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.right())?)
            {
                (Status::Success(Some(i), err), pos) => match (self.f)(state.get_acc(), i) {
                    Some(val) => {
                        merge_errors(&mut state.error, err, &pos);
                        state.set_start(|| pos.start);
                        break (
                            Status::Success(Some(val), state.error()),
                            state.start()..pos.end,
                        );
                    }
                    None => {
                        merge_errors(&mut state.error, err, &pos);
                        state.set_start(|| pos.start);
                    }
                },
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    break (Status::Success(None, state.error()), state.start()..pos.end);
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    state.set_start(|| pos.start);
                    break (Status::Failure(err, true), state.start()..pos.end);
                }
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
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, init: Q, f: F) -> Self {
        Self { inner, init, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.init)
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
    type State = ScanState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.init.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(acc, err), pos) => {
                    state.start = Some(pos.start);
                    state.error = err;
                    state.inner = EitherState::new_right();
                    state.acc = Some(acc);
                }
                (Status::Failure(err, exclusive), pos) => {
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)))
                }
            }
        }

        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.right())?)
            {
                (Status::Success(Some(i), err), pos) => match (self.f)(state.get_acc(), i) {
                    Ok(Some(val)) => {
                        merge_errors(&mut state.error, err, &pos);
                        state.set_start(|| pos.start);
                        break (
                            Status::Success(Some(val), state.error()),
                            state.start()..pos.end,
                        );
                    }
                    Ok(None) => {
                        merge_errors(&mut state.error, err, &pos);
                        state.set_start(|| pos.start);
                    }
                    Err(exp) => {
                        let Range { start, end } = pos;
                        state.set_start(|| start.clone());
                        break (
                            Status::Failure(
                                Error {
                                    expects: exp.into(),
                                    position: start..end.clone(),
                                },
                                true,
                            ),
                            state.start()..end,
                        );
                    }
                },
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    break (Status::Success(None, state.error()), state.start()..pos.end);
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    state.set_start(|| pos.start);
                    break (Status::Failure(err, true), state.start()..pos.end);
                }
            }
        }))
    }
}
