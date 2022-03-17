use futures_core::{Stream, TryStream};
#[cfg(feature = "std")]
use futures_io::AsyncRead;

use super::position::{Locator, PositionedStream};
use super::{InfallibleStream, IteratorStream, SliceStream};

#[cfg(feature = "alloc")]
use super::record::{ExtendRecorder, VecRecorder};
#[cfg(feature = "alloc")]
use super::rewind::BufferedRewinder;

#[cfg(feature = "std")]
use super::ReaderStream;

/// An utility trait to build a stream from various type.
pub trait StreamBuilder: TryStream {
    /// Implements [`Positioned`] to a stream.
    ///
    /// # Examples
    /// ```
    /// # futures::executor::block_on(async {
    /// use somen::stream::{StreamBuilder, position::Positioned};
    /// use futures::stream::TryStreamExt;
    ///
    /// let mut stream = somen::stream::from_slice(b"abc").positioned::<usize>();
    /// // Initial position is 0.
    /// assert_eq!(stream.position(), 0);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'a'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'b'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'c'));
    ///
    /// // The position incremented.
    /// assert_eq!(stream.position(), 3);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), None);
    ///
    /// // The position will not be incremented if the stream have already ended.
    /// assert_eq!(stream.position(), 3);
    /// # });
    /// ```
    ///
    /// [`Positioned`]: crate::stream::position::Positioned
    #[inline]
    fn positioned<P>(self) -> PositionedStream<Self, P>
    where
        P: Locator<Self::Ok> + Default,
        Self: Sized,
    {
        PositionedStream::from(self)
    }

    /// Implements [`Positioned`] to a stream, with a custom initial value.
    ///
    /// # Examples
    /// ```
    /// # futures::executor::block_on(async {
    /// use somen::stream::{StreamBuilder, position::Positioned};
    /// use futures::stream::TryStreamExt;
    ///
    /// let mut stream = somen::stream::from_slice(b"abc").positioned_by(1i32);
    /// // Initial position is 0.
    /// assert_eq!(stream.position(), 1);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'a'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'b'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'c'));
    ///
    /// // The position incremented.
    /// assert_eq!(stream.position(), 4);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), None);
    ///
    /// // The position will not be incremented if the stream have already ended.
    /// assert_eq!(stream.position(), 4);
    /// # });
    /// ```
    ///
    /// [`Positioned`]: crate::stream::position::Positioned
    #[inline]
    fn positioned_by<P>(self, initial: P) -> PositionedStream<Self, P>
    where
        P: Locator<Self::Ok>,
        Self: Sized,
    {
        PositionedStream::new(self, initial)
    }

    /// Implements [`Positioned`] and [`Rewind`] by buffering recent inputs.
    ///
    /// # Examples
    /// ```
    /// # futures::executor::block_on(async {
    /// use somen::stream::{StreamBuilder, position::Positioned, rewind::Rewind};
    /// use futures::stream::TryStreamExt;
    ///
    /// let mut stream = somen::stream::from_slice(b"abc")
    ///     .buffered_rewind();
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'a'));
    ///
    /// let marker = stream.mark_unpin().unwrap();
    ///
    /// assert_eq!(stream.position(), 1);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'b'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'c'));
    /// assert_eq!(stream.try_next().await.unwrap(), None);
    ///
    /// assert_eq!(stream.position(), 3);
    ///
    /// stream.rewind_unpin(marker).unwrap();
    ///
    /// assert_eq!(stream.position(), 1);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'b'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'c'));
    /// assert_eq!(stream.try_next().await.unwrap(), None);
    ///
    /// assert_eq!(stream.position(), 3);
    /// # });
    /// ```
    ///
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    #[inline]
    fn buffered_rewind(self) -> BufferedRewinder<Self>
    where
        Self: Sized,
    {
        BufferedRewinder::from(self)
    }

    /// Implements [`Positioned`] and [`Rewind`] by recording all the output to [`Vec`].
    ///
    /// # Examples
    /// ```
    /// # futures::executor::block_on(async {
    /// use somen::stream::{StreamBuilder, position::Positioned, rewind::Rewind};
    /// use futures::stream::TryStreamExt;
    ///
    /// let mut stream = somen::stream::from_slice(b"abc")
    ///     .record_to_vec();
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'a'));
    ///
    /// let marker = stream.mark_unpin().unwrap();
    ///
    /// assert_eq!(stream.position(), 1);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'b'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'c'));
    /// assert_eq!(stream.try_next().await.unwrap(), None);
    ///
    /// assert_eq!(stream.position(), 3);
    ///
    /// stream.rewind_unpin(marker).unwrap();
    ///
    /// assert_eq!(stream.position(), 1);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'b'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some(b'c'));
    /// assert_eq!(stream.try_next().await.unwrap(), None);
    ///
    /// assert_eq!(stream.position(), 3);
    ///
    /// assert_eq!(stream.into_vec(), b"abc".to_vec());
    /// # });
    /// ```
    ///
    /// [`Vec`]: alloc::vec::Vec
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    #[inline]
    fn record_to_vec(self) -> VecRecorder<Self>
    where
        Self: Sized,
    {
        VecRecorder::from(self)
    }

    /// Records consumed items by using types implements [`Extend`].
    ///
    /// # Examples
    /// ```
    /// # futures::executor::block_on(async {
    /// use somen::stream::StreamBuilder;
    /// use futures::stream::TryStreamExt;
    ///
    /// let mut s = String::new();
    /// let mut stream = somen::stream::from_iter("abc".chars())
    ///     .record_to_extend(&mut s);
    ///
    /// assert_eq!(stream.try_next().await.unwrap(), Some('a'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some('b'));
    /// assert_eq!(stream.try_next().await.unwrap(), Some('c'));
    /// assert_eq!(stream.try_next().await.unwrap(), None);
    ///
    /// assert_eq!(s, String::from("abc"));
    /// # });
    /// ```
    ///
    /// [`Extend`]: core::iter::Extend
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
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

/// A normal(infallible) [`Stream`] into a [`TryStream`].
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use futures::future::FutureExt;
/// use futures::stream::{TryStreamExt, once};
///
/// let mut stream = somen::stream::from_stream(once(async { 'a' }.boxed()));
/// assert_eq!(stream.try_next().await, Ok(Some('a')));
/// assert_eq!(stream.try_next().await, Ok(None));
/// # });
/// ```
#[inline]
pub fn from_stream<S: Stream>(stream: S) -> InfallibleStream<S> {
    InfallibleStream::from(stream)
}

/// An [`Iterator`] with `Item=Result<T, E>` into a [`TryStream`].
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use futures::stream::TryStreamExt;
///
/// let mut stream = somen::stream::from_try_iter(vec![Ok('a'), Err("foo")]);
/// assert_eq!(stream.try_next().await, Ok(Some('a')));
/// assert_eq!(stream.try_next().await, Err("foo"));
/// # });
/// ```
#[inline]
pub fn from_try_iter<I, T, E>(iter: I) -> IteratorStream<I::IntoIter>
where
    I: IntoIterator<Item = Result<T, E>>,
{
    IteratorStream::from(iter.into_iter())
}

/// An [`Iterator`] into a [`TryStream`].
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use futures::stream::TryStreamExt;
///
/// let mut stream = somen::stream::from_iter(Some(1));
/// assert_eq!(stream.try_next().await, Ok(Some(1u8)));
/// assert_eq!(stream.try_next().await, Ok(None));
/// # });
/// ```
#[inline]
pub fn from_iter<I: IntoIterator>(iter: I) -> InfallibleStream<IteratorStream<I::IntoIter>> {
    InfallibleStream::from(IteratorStream::from(iter.into_iter()))
}

/// A reader implements [`AsyncRead`] into a [`TryStream`].
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use futures::stream::TryStreamExt;
/// use futures::io::BufReader;
///
/// let mut stream = somen::stream::from_reader(BufReader::new(b"abc".as_slice()));
/// assert_eq!(stream.try_next().await.unwrap(), Some(b'a'));
/// assert_eq!(stream.try_next().await.unwrap(), Some(b'b'));
/// assert_eq!(stream.try_next().await.unwrap(), Some(b'c'));
/// assert_eq!(stream.try_next().await.unwrap(), None);
/// # });
/// ```
#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
#[inline]
pub fn from_reader<R: AsyncRead>(reader: R) -> ReaderStream<R> {
    ReaderStream::from(reader)
}

/// A slice into a [`TryStream`] implements [`Positioned`] and [`Rewind`].
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use futures::stream::TryStreamExt;
///
/// let mut stream = somen::stream::from_slice(b"abc");
/// assert_eq!(stream.try_next().await.unwrap(), Some(b'a'));
/// assert_eq!(stream.try_next().await.unwrap(), Some(b'b'));
/// assert_eq!(stream.try_next().await.unwrap(), Some(b'c'));
/// assert_eq!(stream.try_next().await.unwrap(), None);
/// # });
/// ```
///
/// [`Positioned`]: crate::stream::position::Positioned
/// [`Rewind`]: crate::stream::rewind::Rewind
#[inline]
pub fn from_slice<T: Clone>(slice: &[T]) -> SliceStream<'_, T> {
    SliceStream::from(slice)
}
