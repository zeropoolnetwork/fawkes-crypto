use std::{
    iter::*,
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice::SliceIndex,
    slice::{Iter, IterMut},
};

use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use typenum::Unsigned;

#[derive(Debug, Clone)]
pub struct SizedVec<T: Sized, L: Unsigned>(pub Vec<T>, pub PhantomData<L>);

impl<T, L: Unsigned> SizedVec<T, L> {
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.0.iter_mut()
    }
}

impl<T: Serialize, L: Unsigned> Serialize for SizedVec<T, L> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>, L: Unsigned> Deserialize<'de> for SizedVec<T, L> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SizedVec<T, L>, D::Error> {
        Vec::<T>::deserialize(deserializer).map(|v| SizedVec::<T, L>(v, PhantomData))
    }
}

impl<T, L: Unsigned> FromIterator<T> for SizedVec<T, L> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let v = Vec::<T>::from_iter(iter);
        assert!(v.len() == L::USIZE, "Wrong length of SizedVec");
        Self(v, PhantomData)
    }
}

impl<T, I: SliceIndex<[T]>, L: Unsigned> Index<I> for SizedVec<T, L> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&*self.0, index)
    }
}

impl<T, I: SliceIndex<[T]>, L: Unsigned> IndexMut<I> for SizedVec<T, L> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut *self.0, index)
    }
}
