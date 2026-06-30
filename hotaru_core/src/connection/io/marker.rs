//! Backend-tagged IO adapter marker.

/// Backend-tagged IO adapter.
///
/// The `Backend` parameter makes each adapter a distinct concrete `Self`
/// type, so non-Tokio IO ecosystems can implement Hotaru IO traits without
/// adding broad blanket impls that overlap with Tokio's compatibility blanket.
pub struct IoCompat<T, Backend> {
    inner: T,
    _backend: core::marker::PhantomData<fn() -> Backend>,
}

impl<T, Backend> IoCompat<T, Backend> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _backend: core::marker::PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
