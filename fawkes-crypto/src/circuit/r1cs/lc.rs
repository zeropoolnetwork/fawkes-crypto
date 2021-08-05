use std::usize;

use crate::ff_uint::{Num, PrimeField};
use linked_list::{Cursor, LinkedList};

#[cfg(feature="borsh_support")]
use crate::borsh::{BorshSerialize, BorshDeserialize};


pub trait AbstractLC<Fr:PrimeField> : Clone + std::fmt::Debug{
    fn add_assign(&mut self, other: &Self);
    fn sub_assign(&mut self, other: &Self);
    fn mul_assign(&mut self, other: &Num<Fr>);
    fn neg(&self)->Self;
    fn to_vec(&self) -> Vec<(Num<Fr>, Index)> ;
    fn new() -> Self;
    fn from_index(index:Index) -> Self;
    fn from_parts(value:Num<Fr>, index:Index) -> Self;
    fn is_empty(&self) -> bool;
    fn as_const(&self) -> Option<Num<Fr>>;
    fn capacity(&self) -> usize;
}


#[derive(Clone, Debug)]
pub struct ZeroLC;

impl<Fr:PrimeField> AbstractLC<Fr> for ZeroLC {
    fn add_assign(&mut self, _: &Self) {}
    fn sub_assign(&mut self, _: &Self) {}
    fn mul_assign(&mut self, _: &Num<Fr>) {}
    fn neg(&self) -> Self {self.clone()}
    fn to_vec(&self) -> Vec<(Num<Fr>, Index)> {vec![]}
    fn new() -> Self {ZeroLC}
    fn from_index(_:Index) -> Self {ZeroLC}
    fn from_parts(_:Num<Fr>, _:Index) -> Self {ZeroLC}
    fn is_empty(&self) -> bool {std::unimplemented!()}
    fn as_const(&self) -> Option<Num<Fr>> {std::unimplemented!()}
    fn capacity(&self) -> usize {0}
}


#[derive(Clone, Debug)]
pub struct LC<Fr:PrimeField>(pub LinkedList<(Num<Fr>, Index)>);

impl<Fr:PrimeField> AbstractLC<Fr> for LC<Fr> {
    fn to_vec(&self) -> Vec<(Num<Fr>, Index)> {
        self.0.iter().cloned().collect()
    }

    fn new() -> Self {
        LC(LinkedList::new())
    }

    fn from_index(index:Index) -> Self {
        Self::from_parts(Num::ONE, index)
    }

    fn from_parts(value:Num<Fr>, index:Index) -> Self {
        let mut res = Self::new();
        res.0.push_back((value, index));
        res
    }

    fn is_empty(&self) -> bool {
        self.0.len()==0
    }

    fn as_const(&self) -> Option<Num<Fr>> {
        if self.0.is_empty() {
            Some(Num::ZERO)
        } else if self.0.len() == 1 {
            let front = self.0.front().unwrap();
            if front.1 == Index::Input(0) {
                Some(front.0)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn capacity(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn add_assign(&mut self, other: &LC<Fr>) {
        let mut cur_a_ll = self.0.cursor();
        for (v, k) in other.0.iter() {
            if ll_lookup(&mut cur_a_ll, *k) == LookupAction::Add {
                let t = cur_a_ll.peek_next().unwrap();
                t.0 += *v;
                if t.0.is_zero() {
                    cur_a_ll.remove();
                }
            } else {
                cur_a_ll.insert((*v, *k))
            }
        }
    }

    #[inline]
    fn sub_assign(&mut self, other: &LC<Fr>) {
        let mut cur_a_ll = self.0.cursor();
        for (v, k) in other.0.iter() {
            if ll_lookup(&mut cur_a_ll, *k) == LookupAction::Add {
                let t = cur_a_ll.peek_next().unwrap();
                t.0 -= *v;
                if t.0.is_zero() {
                    cur_a_ll.remove();
                }
            } else {
                cur_a_ll.insert((-*v, *k))
            }
        } 
    }

    #[inline]
    fn mul_assign(&mut self, other: &Num<Fr>) {
        if other.is_zero() {
            *self = LC::new();
        } else {
            for (v, _) in self.0.iter_mut() {
                *v *= other;
            }
        }
    }

    #[inline]
    fn neg(&self) -> Self {
        let mut res = self.clone();
        for (v, _) in res.0.iter_mut() {
            *v = -*v;
        }
        res
    }

}



#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "borsh_support", derive(BorshSerialize, BorshDeserialize))]
pub enum Index {
    Input(u32),
    Aux(u32)
}

impl core::cmp::Eq for Index {}

impl core::cmp::Ord for Index {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match (self, other) {
            (Index::Input(a), Index::Input(b)) => a.cmp(b),
            (Index::Input(_), Index::Aux(_)) => core::cmp::Ordering::Less,
            (Index::Aux(_), Index::Input(_)) => core::cmp::Ordering::Greater,
            (Index::Aux(a), Index::Aux(b)) => a.cmp(b)
        }
    }
}

impl core::cmp::PartialOrd for Index {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Eq, PartialEq)]
enum LookupAction {
    Add,
    Insert,
}

#[inline]
fn ll_lookup<V, K: PartialEq + PartialOrd>(cur: &mut Cursor<(V, K)>, n: K) -> LookupAction {
    loop {
        match cur.peek_next() {
            Some((_, k)) => {
                if *k == n {
                    return LookupAction::Add;
                } else if *k > n {
                    return LookupAction::Insert;
                }
            }
            None => {
                return LookupAction::Insert;
            }
        }
        cur.seek_forward(1);
    }
}
