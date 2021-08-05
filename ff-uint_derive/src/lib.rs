#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate proc_macro2;

use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{One, ToPrimitive, Zero};
use proc_macro::TokenStream;
use proc_macro_crate::crate_name;
use proc_macro2::Span;
use quote::quote;
use quote::TokenStreamExt;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{parse_macro_input, Expr, ExprLit, Ident, ImplItem, ItemImpl, ItemStruct, Lit};

struct PrimeFieldParamsDef {
    struct_def: ItemStruct,
    impl_block: ItemImpl,
}

impl Parse for PrimeFieldParamsDef {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let struct_def: ItemStruct = input.parse()?;
        let impl_block: ItemImpl = input.parse()?;

        Ok(PrimeFieldParamsDef {
            struct_def,
            impl_block,
        })
    }
}

#[proc_macro]
pub fn construct_primefield_params(input: TokenStream) -> TokenStream {
    let PrimeFieldParamsDef {
        struct_def,
        impl_block,
    } = parse_macro_input!(input as PrimeFieldParamsDef);

    let cratename = Ident::new(
        &crate_name("ff_uint").unwrap_or_else(|_| "ff_uint".to_string()),
        Span::call_site(),
    );

    if let Some((_, name, _)) = &impl_block.trait_ {
        if name.segments.last().unwrap().ident != "PrimeFieldParams" {
            panic!("Invalid trait, expected PrimeFieldParams");
        }
    } else {
        panic!("PrimeFieldParams implementation must be present");
    }

    let self_ty = impl_block.self_ty;
    let repr_ty = impl_block
        .items
        .iter()
        .find_map(|item| {
            if let ImplItem::Type(ty) = item {
                if ty.ident == "Inner" {
                    Some(ty.ty.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Associated type Inner must be specified");

    let modulus: BigUint = fetch_const("MODULUS", &impl_block.items)
        .parse()
        .expect("MODULUS should be a valid number");
    let generator: BigUint = fetch_const("GENERATOR", &impl_block.items)
        .parse()
        .expect("GENERATOR should be a valid number");

    // The arithmetic in this library only works if the modulus*2 is smaller than the backing
    // representation. Compute the number of limbs we need.
    let mut limbs = 1;
    {
        let mod2 = (&modulus) << 1; // modulus * 2
        let mut cur = BigUint::one() << 64; // always 64-bit limbs for now
        while cur < mod2 {
            limbs += 1;
            cur <<= 64;
        }
    }

    let mut gen = proc_macro2::TokenStream::new();

    let (params_impl, sqrt_impl) =
        prime_field_constants_and_sqrt(&cratename, &self_ty, &repr_ty, modulus, limbs, generator);

    let module_name = Ident::new(
        &format!("__generated_{}", struct_def.ident),
        Span::call_site(),
    );
    let prime_field_impl = prime_field_impl(&cratename, &self_ty, &repr_ty, limbs);

    gen.extend(quote! {
        pub use self::#module_name::*;
        mod #module_name {
            use ::#cratename::PrimeFieldParams;
            use ::#cratename::Field;
            use ::#cratename::PrimeField;
            use ::#cratename::Uint;
            #struct_def
            #params_impl
            #prime_field_impl
            #sqrt_impl
        }
    });

    // Return the generated impl
    gen.into()
}

/// Fetch a constant string from an impl block.
fn fetch_const(name: &str, items: &[ImplItem]) -> String {
    let c = items
        .iter()
        .find_map(|item| {
            if let ImplItem::Const(c) = item {
                if c.ident == name {
                    Some(c)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Associated constant MODULUS must be specified");

    match c.expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(ref s),
            ..
        }) => s.value(),
        _ => {
            panic!("Associated constant {} should be a string", name);
        }
    }
}

/// Convert BigUint into a vector of 64-bit limbs.
fn biguint_to_real_u64_vec(mut v: BigUint, limbs: usize) -> Vec<u64> {
    let m = BigUint::one() << 64;
    let mut ret = vec![];

    while v > BigUint::zero() {
        let t: BigUint = &v % &m;
        ret.push(t.to_u64().unwrap());
        v >>= 64;
    }

    while ret.len() < limbs {
        ret.push(0);
    }

    assert!(ret.len() == limbs);

    ret
}

/// Convert BigUint into a tokenized vector of 64-bit limbs.
fn biguint_to_u64_vec(v: BigUint, limbs: usize) -> proc_macro2::TokenStream {
    let ret = biguint_to_real_u64_vec(v, limbs);
    quote!([#(#ret,)*])
}

fn biguint_num_bits(mut v: BigUint) -> u32 {
    let mut bits = 0;

    while v != BigUint::zero() {
        v >>= 1;
        bits += 1;
    }

    bits
}

/// BigUint modular exponentiation by square-and-multiply.
fn exp(base: BigUint, exp: &BigUint, modulus: &BigUint) -> BigUint {
    let mut ret = BigUint::one();

    for i in exp
        .to_bytes_be()
        .into_iter()
        .flat_map(|x| (0..8).rev().map(move |i| (x >> i).is_odd()))
    {
        ret = (&ret * &ret) % modulus;
        if i {
            ret = (ret * &base) % modulus;
        }
    }

    ret
}

#[test]
fn test_exp() {
    assert_eq!(
        exp(
            BigUint::from_str("4398572349857239485729348572983472345").unwrap(),
            &BigUint::from_str("5489673498567349856734895").unwrap(),
            &BigUint::from_str(
                "52435875175126190479447740508185965837690552500527637822603658699938581184513"
            )
            .unwrap()
        ),
        BigUint::from_str(
            "4371221214068404307866768905142520595925044802278091865033317963560480051536"
        )
        .unwrap()
    );
}

fn prime_field_constants_and_sqrt(
    cratename: &Ident,
    name: &syn::Type,
    inner: &syn::Type,
    modulus: BigUint,
    limbs: usize,
    generator: BigUint,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let modulus_num_bits = biguint_num_bits(modulus.clone());

    // The number of bits we should "shave" from a randomly sampled reputation, i.e.,
    // if our modulus is 381 bits and our representation is 384 bits, we should shave
    // 3 bits from the beginning of a randomly sampled 384 bit representation to
    // reduce the cost of rejection sampling.
    let inner_shave_bits = (64 * limbs as u32) - biguint_num_bits(modulus.clone());

    // Compute R = 2**(64 * limbs) mod m
    let r = (BigUint::one() << (limbs * 64)) % &modulus;

    // modulus - 1 = 2^s * t
    let mut s: u32 = 0;
    let mut t = &modulus - BigUint::from_str("1").unwrap();
    while t.is_even() {
        t >>= 1;
        s += 1;
    }

    // Compute 2^s root of unity given the generator
    let root_of_unity = biguint_to_u64_vec(
        (exp(generator.clone(), &t, &modulus) * &r) % &modulus,
        limbs,
    );
    let generator = biguint_to_u64_vec((generator * &r) % &modulus, limbs);

    let mod_minus_1_over_2 =
        biguint_to_u64_vec((&modulus - BigUint::from_str("1").unwrap()) >> 1, limbs);
    let legendre_impl = quote! {
        fn legendre(&self) -> ::#cratename::LegendreSymbol {
            // s = self^((modulus - 1) // 2)
            let s = self.pow(#inner::new(#mod_minus_1_over_2));
            if s.is_zero() {
                ::#cratename::LegendreSymbol::Zero
            } else if s == Self::ONE {
                ::#cratename::LegendreSymbol::QuadraticResidue
            } else {
                ::#cratename::LegendreSymbol::QuadraticNonResidue
            }
        }
    };

    let sqrt_impl =
        if (&modulus % BigUint::from_str("4").unwrap()) == BigUint::from_str("3").unwrap() {
            let mod_minus_3_over_4 =
                biguint_to_u64_vec((&modulus - BigUint::from_str("3").unwrap()) >> 2, limbs);

            // Compute -R as (m - r)
            let rneg = biguint_to_u64_vec(&modulus - &r, limbs);

            quote! {
                impl ::#cratename::SqrtField for #name {
                    #legendre_impl

                    fn sqrt(&self) -> Option<Self> {
                        // Shank's algorithm for q mod 4 = 3
                        // https://eprint.iacr.org/2012/685.pdf (page 9, algorithm 2)

                        let mut a1 = self.pow(#inner::new(#mod_minus_3_over_4));

                        let mut a0 = a1;
                        a0 = a0.square().wrapping_mul(*self);

                        if a0.0 == #inner::new(#rneg) {
                            None
                        } else {
                            Some(a1.wrapping_mul(*self))
                        }
                    }
                }
            }
        } else if (&modulus % BigUint::from_str("16").unwrap()) == BigUint::from_str("1").unwrap() {
            let t_plus_1_over_2 = biguint_to_u64_vec((&t + BigUint::one()) >> 1, limbs);
            let t = biguint_to_u64_vec(t, limbs);

            quote! {
                impl ::#cratename::SqrtField for #name {
                    #legendre_impl

                    fn sqrt(&self) -> Option<Self> {
                        // Tonelli-Shank's algorithm for q mod 16 = 1
                        // https://eprint.iacr.org/2012/685.pdf (page 12, algorithm 5)

                        match self.legendre() {
                            ::#cratename::LegendreSymbol::Zero => Some(*self),
                            ::#cratename::LegendreSymbol::QuadraticNonResidue => None,
                            ::#cratename::LegendreSymbol::QuadraticResidue => {
                                let mut c = #name(Self::ROOT_OF_UNITY);
                                let mut r = self.pow(#inner::new(#t_plus_1_over_2));
                                let mut t = self.pow(#inner::new(#t));
                                let mut m = <Self as PrimeFieldParams>::S;

                                while t != Self::ONE {
                                    let mut i = 1;
                                    {
                                        let mut t2i = t;
                                        t2i=t2i.square();
                                        loop {
                                            if t2i == Self::ONE {
                                                break;
                                            }
                                            t2i= t2i.square();
                                            i += 1;
                                        }
                                    }

                                    for _ in 0..(m - i - 1) {
                                        c=c.square();
                                    }
                                    r=r.wrapping_mul(c);
                                    c=c.square();
                                    t=t.wrapping_mul(c);
                                    m = i;
                                }

                                Some(r)
                            }
                        }
                    }
                }
            }
        } else {
            quote! {}
        };

    // Compute R^2 mod m
    let r2 = biguint_to_u64_vec((&r * &r) % &modulus, limbs);

    let r = biguint_to_u64_vec(r, limbs);
    let modulus = biguint_to_real_u64_vec(modulus, limbs);

    // Compute -m^-1 mod 2**64 by exponentiating by totient(2**64) - 1
    let mut inv = 1u64;
    for _ in 0..63 {
        inv = inv.wrapping_mul(inv);
        inv = inv.wrapping_mul(modulus[0]);
    }
    inv = inv.wrapping_neg();

    (
        quote! {
            impl ::#cratename::PrimeFieldParams for #name {
                type Inner = #inner;

                /// This is the modulus m of the prime field
                const MODULUS: #inner = #inner::new([#(#modulus,)*]);

                /// The number of bits needed to represent the modulus.
                const MODULUS_BITS: u32 = #modulus_num_bits;

                /// The number of bits that must be shaved from the beginning of
                /// the representation when randomly sampling.
                const REPR_SHAVE_BITS: u32 = #inner_shave_bits;

                /// 2^{limbs*64} mod m
                const R: #inner = #inner::new(#r);

                /// 2^{limbs*64*2} mod m
                const R2: #inner = #inner::new(#r2);

                /// -(m^{-1} mod m) mod m
                const INV: u64 = #inv;

                /// Multiplicative generator of `MODULUS` - 1 order, also quadratic
                /// nonresidue.
                const GENERATOR: #inner = #inner::new(#generator);

                /// 2^s * t = MODULUS - 1 with t odd
                const S: u32 = #s;

                /// 2^s root of unity computed by GENERATOR^t
                const ROOT_OF_UNITY: #inner = #inner::new(#root_of_unity);
            }
        },
        sqrt_impl,
    )
}

/// Implement PrimeField for the derived type.
fn prime_field_impl(
    cratename: &Ident,
    name: &syn::Type,
    inner: &syn::Type,
    limbs: usize,
) -> proc_macro2::TokenStream {
    // Returns r{n} as an ident.
    fn get_temp(n: usize) -> syn::Ident {
        syn::Ident::new(&format!("r{}", n), proc_macro2::Span::call_site())
    }

    // The parameter list for the mont_reduce() internal method.
    // r0: u64, mut r1: u64, mut r2: u64, ...
    let mut mont_paramlist = proc_macro2::TokenStream::new();
    mont_paramlist.append_separated(
        (0..(limbs * 2)).map(|i| (i, get_temp(i))).map(|(i, x)| {
            if i != 0 {
                quote! {mut #x: u64}
            } else {
                quote! {#x: u64}
            }
        }),
        proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
    );

    // Implement montgomery reduction for some number of limbs
    fn mont_impl(cratename: &Ident, limbs: usize) -> proc_macro2::TokenStream {
        let mut gen = proc_macro2::TokenStream::new();

        for i in 0..limbs {
            {
                let temp = get_temp(i);
                gen.extend(quote! {
                    let k = #temp.wrapping_mul(Self::INV);
                    let mut carry = 0;
                    ::#cratename::mac_with_carry(#temp, k, Self::MODULUS.0[0], &mut carry);
                });
            }

            for j in 1..limbs {
                let temp = get_temp(i + j);
                gen.extend(quote! {
                    #temp = ::#cratename::mac_with_carry(#temp, k, Self::MODULUS.0[#j], &mut carry);
                });
            }

            let temp = get_temp(i + limbs);

            if i == 0 {
                gen.extend(quote! {
                    #temp = ::#cratename::adc(#temp, 0, &mut carry);
                });
            } else {
                gen.extend(quote! {
                    #temp = ::#cratename::adc(#temp, carry2, &mut carry);
                });
            }

            if i != (limbs - 1) {
                gen.extend(quote! {
                    let carry2 = carry;
                });
            }
        }

        gen.extend(quote! {
            let mut res = Self::ZERO;
        });

        for i in 0..limbs {
            let temp = get_temp(limbs + i);

            gen.extend(quote! {
                res.0 .0[#i] = #temp;
            });
        }

        gen.extend(quote! {
            res
        });

        gen
    }

    fn sqr_impl(
        cratename: &Ident,
        a: proc_macro2::TokenStream,
        limbs: usize,
    ) -> proc_macro2::TokenStream {
        let mut gen = proc_macro2::TokenStream::new();

        for i in 0..(limbs - 1) {
            gen.extend(quote! {
                let mut carry = 0;
            });

            for j in (i + 1)..limbs {
                let temp = get_temp(i + j);
                if i == 0 {
                    gen.extend(quote! {
                        let #temp = ::#cratename::mac_with_carry(0, (#a.0).0[#i], (#a.0).0[#j], &mut carry);
                    });
                } else {
                    gen.extend(quote!{
                        let #temp = ::#cratename::mac_with_carry(#temp, (#a.0).0[#i], (#a.0).0[#j], &mut carry);
                    });
                }
            }

            let temp = get_temp(i + limbs);

            gen.extend(quote! {
                let #temp = carry;
            });
        }

        for i in 1..(limbs * 2) {
            let temp0 = get_temp(limbs * 2 - i);
            let temp1 = get_temp(limbs * 2 - i - 1);

            if i == 1 {
                gen.extend(quote! {
                    let #temp0 = #temp1 >> 63;
                });
            } else if i == (limbs * 2 - 1) {
                gen.extend(quote! {
                    let #temp0 = #temp0 << 1;
                });
            } else {
                gen.extend(quote! {
                    let #temp0 = (#temp0 << 1) | (#temp1 >> 63);
                });
            }
        }

        gen.extend(quote! {
            let mut carry = 0;
        });

        for i in 0..limbs {
            let temp0 = get_temp(i * 2);
            let temp1 = get_temp(i * 2 + 1);
            if i == 0 {
                gen.extend(quote! {
                    let #temp0 = ::#cratename::mac_with_carry(0, (#a.0).0[#i], (#a.0).0[#i], &mut carry);
                });
            } else {
                gen.extend(quote!{
                    let #temp0 = ::#cratename::mac_with_carry(#temp0, (#a.0).0[#i], (#a.0).0[#i], &mut carry);
                });
            }

            gen.extend(quote! {
                let #temp1 = ::#cratename::adc(#temp1, 0, &mut carry);
            });
        }

        let mut mont_calling = proc_macro2::TokenStream::new();
        mont_calling.append_separated(
            (0..(limbs * 2)).map(get_temp),
            proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
        );

        gen.extend(quote! {
            Self::mont_reduce(#mont_calling)
        });

        gen
    }

    fn mul_impl(
        cratename: &Ident,
        a: proc_macro2::TokenStream,
        b: proc_macro2::TokenStream,
        limbs: usize,
    ) -> proc_macro2::TokenStream {
        let mut gen = proc_macro2::TokenStream::new();

        for i in 0..limbs {
            gen.extend(quote! {
                let mut carry = 0;
            });

            for j in 0..limbs {
                let temp = get_temp(i + j);

                if i == 0 {
                    gen.extend(quote! {
                        let #temp = ::#cratename::mac_with_carry(0, (#a.0).0[#i], (#b.0).0[#j], &mut carry);
                    });
                } else {
                    gen.extend(quote!{
                        let #temp = ::#cratename::mac_with_carry(#temp, (#a.0).0[#i], (#b.0).0[#j], &mut carry);
                    });
                }
            }

            let temp = get_temp(i + limbs);

            gen.extend(quote! {
                let #temp = carry;
            });
        }

        let mut mont_calling = proc_macro2::TokenStream::new();
        mont_calling.append_separated(
            (0..(limbs * 2)).map(get_temp),
            proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
        );

        gen.extend(quote! {
            Self::mont_reduce(#mont_calling)
        });

        gen
    }

    let squaring_impl = sqr_impl(cratename, quote! {self}, limbs);
    let multiply_impl = mul_impl(cratename, quote! {self}, quote! {other}, limbs);
    let montgomery_impl = mont_impl(cratename, limbs);

    // (self.0).0[0], (self.0).0[1], ..., 0, 0, 0, 0, ...
    let mut into_repr_params = proc_macro2::TokenStream::new();
    into_repr_params.append_separated(
        (0..limbs)
            .map(|i| quote! { (self.0).0[#i] })
            .chain((0..limbs).map(|_| quote! {0})),
        proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
    );

    let top_limb_index = limbs - 1;

    quote! {
        impl ::std::marker::Copy for #name { }

        impl ::std::clone::Clone for #name {
            fn clone(&self) -> #name {
                *self
            }
        }

        impl ::std::cmp::PartialEq for #name {
            fn eq(&self, other: &#name) -> bool {
                self.0 == other.0
            }
        }

        impl ::std::cmp::Eq for #name { }

        impl ::std::fmt::Debug for #name
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}({:?})", stringify!(#name), self.to_uint())
            }
        }

        impl Default for #name {
            fn default() -> Self {
                #name(<#name as PrimeFieldParams>::Inner::default())
            }
        }

        impl std::str::FromStr for #name {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let uint = <<#name as PrimeFieldParams>::Inner as std::str::FromStr>::from_str(s)?;
                Self::from_uint(uint)
                    .ok_or("non-canonical input")
            }
        }

        impl From<&'static str> for #name {
            fn from(s: &'static str) -> Self {
                let uint = <<#name as PrimeFieldParams>::Inner as From<&'static str>>::from(s);
                #name::from_uint(uint).expect("non-canonical input")
            }
        }

        #[cfg(feature = "borsh_support")]
        impl #cratename::borsh::ser::BorshSerialize for #name {
            fn serialize<W: #cratename::borsh::maybestd::io::Write>(&self, writer: &mut W) -> #cratename::borsh::maybestd::io::Result<()> {
                let uint = self.to_uint();
                uint.serialize(writer)
            }
        }

        #[cfg(feature = "borsh_support")]
        impl #cratename::borsh::de::BorshDeserialize for #name {
            fn deserialize(buf: &mut &[u8]) -> #cratename::borsh::maybestd::io::Result<Self> {
                let uint = <<#name as PrimeFieldParams>::Inner as #cratename::borsh::de::BorshDeserialize>::deserialize(buf)?;
                Self::from_uint(uint)
                    .ok_or(#cratename::borsh::maybestd::io::Error::from(#cratename::borsh::maybestd::io::ErrorKind::InvalidData))
            }
        }

        /// Elements are ordered lexicographically.
        impl Ord for #name {
            #[inline(always)]
            fn cmp(&self, other: &#name) -> ::std::cmp::Ordering {
                self.to_uint().cmp(&other.to_uint())
            }
        }

        impl PartialOrd for #name {
            #[inline(always)]
            fn partial_cmp(&self, other: &#name) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", self.to_uint())
            }
        }

        impl From<#name> for #inner {
            fn from(e: #name) -> #inner {
                e.to_uint()
            }
        }

        impl ::#cratename::PrimeField for #name {
            fn from_mont_uint_unchecked(r: #inner) -> Self {
                #name(r)
            }

            fn from_mont_uint(r: #inner) -> Option<Self> {
                let r = #name(r);
                if r.is_valid() {Some(r)} else {None}
            }

            fn from_uint_unchecked(r: #inner) -> Self {
                let r = #name(r);
                r.wrapping_mul(#name(Self::R2))
            }

            fn from_uint(r: #inner) -> Option<Self> {
                let r = #name(r);
                if r.is_valid() {
                    Some(r.wrapping_mul(#name(Self::R2)))
                } else {
                    None
                }
            }

            fn to_uint(&self) -> Self::Inner {
                Self::mont_reduce(#into_repr_params).0
            }

            fn to_mont_uint(&self) -> Self::Inner {
                self.0
            }

            #[inline]
            fn as_mont_uint(&self) -> &Self::Inner {
                let &Self(ref res) = self;
                res
            }

            #[inline]
            fn as_mont_uint_mut(&mut self) -> &mut Self::Inner {
                let &mut Self(ref mut res) = self;
                res
            }
        }

        impl ::#cratename::Field for #name {
            const ZERO: #name = #name(#inner::ZERO);
            const ONE: #name = #name(Self::R);

            /// Computes a uniformly random element using rejection sampling.
            #[cfg(feature = "rand_support")]
            fn random<R: ::#cratename::rand::Rng + ?Sized>(rng: &mut R) -> Self {
                loop {
                    let mut tmp = {
                        let mut repr = [0u64; #limbs];
                        for i in 0..#limbs {
                            repr[i] = rng.next_u64();
                        }
                        #name(#inner::new(repr))
                    };

                    // Mask away the unused most-significant bits.
                    tmp.0.as_inner_mut()[#top_limb_index] &= 0xffffffffffffffff >> Self::REPR_SHAVE_BITS;

                    if tmp.is_valid() {
                        return tmp
                    }
                }
            }

            #[inline]
            fn is_zero(&self) -> bool {
                self.0.is_zero()
            }

            #[inline]
            fn wrapping_add(self, other: #name) -> Self {
                #name(self.0.unchecked_add(other.0)).reduced()
            }

            #[inline]
            fn double(self) -> Self {
                #name(self.0.unchecked_shl(1)).reduced()
            }

            #[inline]
            fn wrapping_sub(self, other: #name) -> Self {
                #name(if other.0 > self.0 {
                    self.0.unchecked_add(Self::MODULUS.unchecked_sub(other.0))
                } else {
                    self.0.unchecked_sub(other.0)
                })
            }

            #[inline]
            fn wrapping_neg(self) -> Self {
                if self.is_zero() {
                    self
                } else {
                    #name(Self::MODULUS.unchecked_sub(self.0))
                }
            }

            fn checked_inv(self) -> Option<Self> {
                if self.is_zero() {
                    None
                } else {
                    // Guajardo Kumar Paar Pelzl
                    // Efficient Software-Implementation of Finite Fields with Applications to Cryptography
                    // Algorithm 16 (BEA for Inversion in Fp)

                    let one = #inner::from(1);

                    let mut u = self.0;
                    let mut v = Self::MODULUS;
                    let mut b = #name(Self::R2); // Avoids unnecessary reduction step.
                    let mut c = Self::ZERO;

                    while u != one && v != one {
                        while u.is_even() {
                            u = u.unchecked_shr(1);

                            if b.0.is_even() {
                                b.0 = b.0.unchecked_shr(1);
                            } else {
                                b.0 = b.0.unchecked_add(Self::MODULUS);
                                b.0 = b.0.unchecked_shr(1);
                            }
                        }

                        while v.is_even() {
                            v = v.unchecked_shr(1);

                            if c.0.is_even() {
                                c.0 = c.0.unchecked_shr(1);
                            } else {
                                c.0 = c.0.unchecked_add(Self::MODULUS);
                                c.0 = c.0.unchecked_shr(1);
                            }
                        }

                        if v < u {
                            u = u.unchecked_sub(v);
                            b = b.wrapping_sub(c);
                        } else {
                            v = v.unchecked_sub(u);
                            c = c.wrapping_sub(b);
                        }
                    }

                    if u == one {
                        Some(b)
                    } else {
                        Some(c)
                    }
                }
            }

            #[inline(always)]
            fn frobenius_map(self, _: usize) -> Self {
                self
            }

            #[inline]
            fn wrapping_mul(self, other: #name) -> Self
            {
                #multiply_impl
            }

            #[inline]
            fn square(self) -> Self
            {
                #squaring_impl
            }
        }

        impl #name {
            /// Determines if the element is really in the field. This is only used
            /// internally.
            #[inline(always)]
            fn is_valid(&self) -> bool {
                self.0 < Self::MODULUS
            }

            /// Subtracts the modulus from this element if this element is not in the
            /// field. Only used interally.
            #[inline]
            fn reduced(self) -> Self {
                if self.is_valid() {
                    self
                } else {
                    #name(self.0.unchecked_sub(Self::MODULUS))
                }
            }

            #[inline]
            fn mont_reduce(
                #mont_paramlist
            ) -> Self
            {
                // The Montgomery reduction here is based on Algorithm 14.32 in
                // Handbook of Applied Cryptography
                // <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.

                #montgomery_impl.reduced()
            }
        }
    }
}
