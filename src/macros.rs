#![allow(unused_macros)]

macro_rules! forward_val_assign_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res {
            #[inline]
            fn $method(&mut self, other: $res2) {
                self.$method(&other);
            }
        }
    };
}



macro_rules! forward_unop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp for &'macro_lifetime $res {
            type Output = $res;
            #[inline]
            fn $method(self) -> $res {
                self.clone().$method()
            }
        }
    };
}



macro_rules! forward_val_val_binop_commutative_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                // forward to val-ref, with the larger capacity as val
                if self.capacity() >= other.capacity() {
                    $imp::$method(self, &other)
                } else {
                    $imp::$method(other, &self)
                }
            }
        }
    };
}


macro_rules! forward_ref_val_binop_commutative_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<$res> for &'macro_lifetime $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                // reverse, forward to val-ref
                $imp::$method(other, self)
            }
        }
    };
}


macro_rules! forward_ref_ref_binop_commutative_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'macro_lifetime_b $res> for &'macro_lifetime_a $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                // forward to val-ref, choosing the larger to clone
                if self.capacity() >= other.capacity() {
                    $imp::$method(self.clone(), other)
                } else {
                    $imp::$method(other.clone(), self)
                }
            }
        }
    };
}

macro_rules! forward_val_val_binop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res {
            type Output = $res3;

            #[inline]
            fn $method(self, other: $res2) -> $res3 {
                // forward to val-ref
                $imp::$method(self, &other)
            }
        }
    };
}

macro_rules! forward_ref_val_binop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for &'macro_lifetime $res {
            type Output = $res3;

            #[inline]
            fn $method(self, other: $res2) -> $res3 {
                // forward to ref-ref
                $imp::$method(self, &other)
            }
        }
    };
}

macro_rules! forward_val_ref_binop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'a $res> for $res {
            type Output = $res3;

            #[inline]
            fn $method(self, other: &$res2) -> $res3 {
                // forward to ref-ref
                $imp::$method(&self, other)
            }
        }
    };
}

macro_rules! forward_ref_ref_binop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'macro_lifetime_b $res2> for &'macro_lifetime_a $res {
            type Output = $res3;

            #[inline]
            fn $method(self, other: &$res2) -> $res3 {
                // forward to val-ref
                $imp::$method(self.clone(), other)
            }
        }
    };
}


macro_rules! forward_all_binop_to_val_ref_commutative_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        forward_val_val_binop_commutative_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp for $res, $method);
        forward_ref_val_binop_commutative_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp for $res, $method);
        forward_ref_ref_binop_commutative_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp for $res, $method);
    };
}


macro_rules! forward_all_binop_to_val_ref_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        forward_val_val_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
        forward_ref_val_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
        forward_ref_ref_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
    };
}

macro_rules! forward_all_binop_to_ref_ref_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        forward_val_val_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
        forward_val_ref_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
        forward_ref_val_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
    };
}

macro_rules! swap_commutative_val_val {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res> for $res2 {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}

macro_rules! swap_commutative_val_ref {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'macro_lifetime $res> for $res2 {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}

macro_rules! swap_commutative_ref_val {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<$res> for &'macro_lifetime $res2 {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}


macro_rules! swap_commutative_ref_ref {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'macro_lifetime_b $res> for &'macro_lifetime_a $res2 {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}


macro_rules! swap_commutative {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        swap_commutative_val_val!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method);
        swap_commutative_val_ref!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method);
        swap_commutative_ref_val!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method);
        swap_commutative_ref_ref!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method);
    };
}

#[macro_export]
macro_rules! num {
    ($x:expr) => {
        $crate::native::num::Num::from($x)
    };
}

// #[macro_export]
// macro_rules! collect_array {
//     ([$t:ty;$size:expr], $iter:expr) => {
//         unsafe {
//             let mut data: [std::mem::MaybeUninit<$t>; $size] = std::mem::MaybeUninit::uninit().assume_init();
//             let mut it = $iter;
//             for i in 0..$size {
//                 match it.next() {
//                     Some(v) => {
//                         data[i] = std::mem::MaybeUninit::new(v);
//                     },
//                     _ => panic!("Not enough elements in iterator")
//                 }
//             }

//             if it.next().is_some() {
//                 panic!("Too much elements in iterator");
//             }

//             let res = std::ptr::read(data.as_ptr() as *const [$t;$size]);
//             std::mem::forget(data);
//             res
//         }
//     };
// }

// #[macro_export]
// macro_rules! collect_opt_array {
//     ([$t:ty;$size:expr], $iter:expr) => {
//         unsafe {
//             let mut data: [std::mem::MaybeUninit<$t>; $size] = std::mem::MaybeUninit::uninit().assume_init();
//             let mut it = $iter;
//             for i in 0..$size {
//                 match it.next() {
//                     Some(v) => {
//                         data[i] = std::mem::MaybeUninit::new(v?);
//                     },
//                     _ => panic!("Not enough elements in iterator")
//                 }
//             }

//             if it.next().is_some() {
//                 panic!("Too much elements in iterator");
//             }

//             let res = std::ptr::read(data.as_ptr() as *const [$t;$size]);
//             std::mem::forget(data);
//             Some(res)
//         }
//     };
// }