use crate::ff_uint::{Num, PrimeField};
use linked_list::{Cursor, LinkedList};
use std::ops::{AddAssign, SubAssign, MulAssign};

#[derive(Clone, Debug)]
pub struct LC<Fr:PrimeField>(pub LinkedList<(Num<Fr>, Index)>);

impl<Fr:PrimeField> LC<Fr> {
    pub fn to_vec(&self) -> Vec<(Num<Fr>, Index)> {
        self.0.iter().cloned().collect()
    }

    pub fn new() -> Self {
        LC(LinkedList::new())
    }

    pub fn from_index(index:Index) -> Self {
        Self::from_parts(Num::ONE, index)
    }

    pub fn from_parts(value:Num<Fr>, index:Index) -> Self {
        let mut res = Self::new();
        res.0.push_back((value, index));
        res
    }

    pub fn is_empty(&self) -> bool {
        self.0.len()==0
    }
}
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Index {
    Input(usize),
    Aux(usize)
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


impl<'l, Fr:PrimeField> AddAssign<&'l LC<Fr>> for LC<Fr> {
    #[inline]
    fn add_assign(&mut self, other: &'l LC<Fr>) {
        if !other.is_empty() {
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
    }
}

impl<'l, Fr:PrimeField> SubAssign<&'l LC<Fr>> for LC<Fr> {
    #[inline]
    fn sub_assign(&mut self, other: &'l LC<Fr>) {
        if !other.is_empty() {
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
    }
}

impl<'l, Fr:PrimeField> MulAssign<&'l Num<Fr>> for LC<Fr> {
    #[inline]
    fn mul_assign(&mut self, other: &'l Num<Fr>) {
        if other.is_zero() {
            *self = LC::new();
        } else {
            for (v, _) in self.0.iter_mut() {
                *v *= other;
            }
        }
    }
}