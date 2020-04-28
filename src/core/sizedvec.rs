use typenum::Unsigned;
use std::iter::*;
use std::marker::PhantomData;
use std::slice::{Iter, IterMut};

use core::slice::SliceIndex;
use std::ops::{Index, IndexMut};


#[derive(Debug, Clone)]
pub struct SizedVec<T:Sized, L:Unsigned>(pub Vec<T>, pub PhantomData<L>);

impl<T, L:Unsigned> SizedVec<T,L> {
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.0.iter_mut()
    }

}

impl<T, L:Unsigned> FromIterator<T> for SizedVec<T, L> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let v = Vec::<T>::from_iter(iter);
        assert!(v.len() == L::USIZE, "Wrong length of SizedVec");
        Self(v, PhantomData)
    }
}


impl<T, I: SliceIndex<[T]>, L:Unsigned> Index<I> for SizedVec<T, L> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&*self.0, index)
    }
}


impl<T, I: SliceIndex<[T]>, L:Unsigned> IndexMut<I> for SizedVec<T, L> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut *self.0, index)
    }
}