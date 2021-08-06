use std::{
    iter::*,
    ops::{Index, IndexMut},
    slice::SliceIndex,
    slice::{Iter, IterMut},
    self
};


#[cfg(feature = "borsh_support")]
use crate::borsh::{BorshSerialize, BorshDeserialize};

#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct SizedVec<T: Sized, const L: usize>([T; L]);

impl<T, const L: usize> SizedVec<T, L> {
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
impl<T: Serialize, const L: usize> Serialize for SizedVec<T, L> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde_support")]
impl<'de, T: Deserialize<'de>, const L: usize> Deserialize<'de> for SizedVec<T, L> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SizedVec<T, L>, D::Error> {
        Vec::<T>::deserialize(deserializer).map(SizedVec::<T, L>::from_iter)
    }
}


#[cfg(feature = "borsh_support")]
impl<T: BorshSerialize, const L: usize> BorshSerialize for SizedVec<T, L> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        for i in 0..L {
            self.0[i].serialize(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "borsh_support")]
impl<T: BorshDeserialize, const L: usize> BorshDeserialize for SizedVec<T, L> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
        (0..L).map(|_| T::deserialize(buf)).collect()
    }
}

// This is a workaround for lack of implementation of FromIterator for [T; N]
// Relevant issue: https://github.com/rust-lang/rust/issues/81615
impl<T, const L: usize> FromIterator<T> for SizedVec<T, L> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let mut data: [std::mem::MaybeUninit<T>; L] = unsafe {
            std::mem::MaybeUninit::uninit().assume_init()
        };

        for elem in &mut data[..] {
            let src_elem = iter.next().expect("iterator is shorter than expected");
            unsafe { std::ptr::write(elem.as_mut_ptr(), src_elem); }
        }

        SizedVec(unsafe { std::mem::transmute_copy::<_, [T; L]>(&data) })
    }
}

impl<T, I: SliceIndex<[T]>, const L: usize> Index<I> for SizedVec<T, L> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.0, index)
    }
}

impl<T, I: SliceIndex<[T]>, const L: usize> IndexMut<I> for SizedVec<T, L> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.0, index)
    }
}
