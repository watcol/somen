use futures_core::{Stream, TryStream};

#[cfg(feature = "std")]
use super::{rewind::SeekRewinder, ReaderStream};
#[cfg(feature = "std")]
use futures_io::{AsyncRead, AsyncSeek};

#[cfg(feature = "alloc")]
use super::{record::VecRecorder, rewind::BufferedRewinder};

use super::{
    position::{CastPositioner, NopPositioner, Positioned, Positioner},
    record::ExtendRecorder,
    InfallibleStream, IteratorStream, SliceStream,
};

/// An utility trait to build a stream from various type.
pub trait StreamBuilder: TryStream {
    /// A normal(infallible) [`Stream`] into a [`TryStream`].
    ///
    /// [`Stream`]: futures_core::stream::Stream
    /// [`TryStream`]: futures_core::stream::TryStream
    #[inline]
    fn from_stream<S: Stream>(stream: S) -> InfallibleStream<S> {
        InfallibleStream::from(stream)
    }

    /// An [`Iterator`] with `Item=Result<T, E>` into a [`TryStream`].
    ///
    /// [`Iterator`]: core::iter::Iterator
    /// [`TryStream`]: futures_core::stream::TryStream
    #[inline]
    fn from_try_iter<I, T, E>(iter: I) -> IteratorStream<I>
    where
        I: Iterator<Item = Result<T, E>>,
    {
        IteratorStream::from(iter)
    }

    /// An [`Iterator`] into a [`TryStream`].
    ///
    /// [`Iterator`]: core::iter::Iterator
    /// [`TryStream`]: futures_core::stream::TryStream
    #[inline]
    fn from_iter<I: Iterator>(iter: I) -> InfallibleStream<IteratorStream<I>> {
        InfallibleStream::from(IteratorStream::from(iter))
    }

    /// A reader implements [`AsyncRead`] into a [`TryStream`].
    ///
    /// [`AsyncRead`]: futures_io::AsyncRead
    /// [`TryStream`]: futures_core::stream::TryStream
    #[cfg(feature = "std")]
    #[cfg_attr(all(doc, feature = "unstable"), doc(cfg(feature = "std")))]
    #[inline]
    fn from_reader<R: AsyncRead>(reader: R) -> ReaderStream<R> {
        ReaderStream::from(reader)
    }

    /// A slice into a [`TryStream`] implements [`Positioned`] and [`Rewind`].
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    #[inline]
    fn from_slice<T: Clone>(slice: &[T]) -> SliceStream<'_, T> {
        SliceStream::from(slice)
    }

    /// Implement [`Positioned`] to a stream.
    ///
    /// [`Positioned`]: crate::stream::position::Positioned
    #[inline]
    fn positioned(self) -> Positioner<Self>
    where
        Self: Sized,
    {
        Positioner::from(self)
    }

    /// Implement [`Positioned`] by `Position = ()` to a stream.
    ///
    /// [`Positioned`]: crate::stream::position::Positioned
    #[inline]
    fn not_positioned(self) -> NopPositioner<Self>
    where
        Self: Sized,
    {
        NopPositioner::from(self)
    }

    /// Casting the type of [`Positioned`].
    ///
    /// [`Positioned`]: crate::stream::position::Positioned
    #[inline]
    fn cast_positon<T>(self) -> CastPositioner<Self, T>
    where
        Self: Positioned + Sized,
        T: TryFrom<Self::Position>,
    {
        CastPositioner::from(self)
    }

    /// Implement [`Positioned`] and [`Rewind`] by buffering recent inputs.
    ///
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    #[cfg(feature = "alloc")]
    #[cfg_attr(all(doc, feature = "unstable"), doc(cfg(feature = "alloc")))]
    #[inline]
    fn buffered_rewind(self) -> BufferedRewinder<Self>
    where
        Self: Sized,
    {
        BufferedRewinder::from(self)
    }

    /// Implement [`Positioned`] and [`Rewind`] using [`AsyncSeek`] trait.
    ///
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    /// [`AsyncSeek`]: futures_io::AsyncSeek
    #[cfg(feature = "std")]
    #[cfg_attr(all(doc, feature = "unstable"), doc(cfg(feature = "std")))]
    #[inline]
    fn seek_rewind(self) -> SeekRewinder<Self>
    where
        Self: Sized + AsyncSeek,
    {
        SeekRewinder::from(self)
    }

    /// Recording stream outputs into a [`Vec`] and implements [`Positioned`], [`Rewind`].
    ///
    /// [`Vec`]: alloc::vec::Vec
    /// [`Positoned`]: crate::stream::position::Positoned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    #[cfg(feature = "alloc")]
    #[cfg_attr(all(doc, feature = "unstable"), doc(cfg(feature = "alloc")))]
    #[inline]
    fn record_to_vec(self) -> VecRecorder<Self>
    where
        Self: Sized,
    {
        VecRecorder::from(self)
    }

    /// Recording stream outputs into any types implements [`Extend`].
    ///
    /// [`Extend`]: core::iter::Extend
    #[inline]
    fn record_to_extend<E>(self, extend: &mut E) -> ExtendRecorder<'_, Self, E>
    where
        Self: Sized,
        E: Extend<Self::Ok>,
    {
        ExtendRecorder::new(self, extend)
    }
}

impl<T: TryStream> StreamBuilder for T {}
