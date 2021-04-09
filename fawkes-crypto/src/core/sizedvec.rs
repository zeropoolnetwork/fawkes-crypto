use std::{
    iter::*,
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice::SliceIndex,
    slice::{Iter, IterMut},
    self
};

use crate::typenum::Unsigned;

#[cfg(feature = "borsh_support")]
use crate::borsh::{BorshSerialize, BorshDeserialize};

#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct SizedVec<T: Sized, L: Unsigned>(Vec<T>, PhantomData<L>);

impl<T: Clone, L: Unsigned> SizedVec<T, L> {
    pub fn from_slice(slice:&[T]) -> Self {
        assert!(slice.len() == L::USIZE, "Wrong length of SizedVec");
        Self(slice.to_vec(), PhantomData)
    }
}

impl<T, L: Unsigned> SizedVec<T, L> {
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.0.iter_mut()
    }
}

#[cfg(feature = "serde_support")]
impl<T: Serialize, L: Unsigned> Serialize for SizedVec<T, L> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde_support")]
impl<'de, T: Deserialize<'de>, L: Unsigned> Deserialize<'de> for SizedVec<T, L> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SizedVec<T, L>, D::Error> {
        Vec::<T>::deserialize(deserializer).map(|v| SizedVec::<T, L>(v, PhantomData))
    }
}


#[cfg(feature = "borsh_support")]
impl<T: BorshSerialize, L: Unsigned> BorshSerialize for SizedVec<T, L> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        for i in 0..L::USIZE {
            self.0[i].serialize(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "borsh_support")]
impl<T: BorshDeserialize, L: Unsigned> BorshDeserialize for SizedVec<T, L> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
        (0..L::USIZE).map(|_| T::deserialize(buf)).collect()
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
