extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse::Error as ParseError;

mod signal;

fn process(this: proc_macro2::TokenStream) -> TokenStream {
    this.into()
}

macro_rules! create_derive(
    ($mod_:ident, $trait_:ident, $fn_name: ident $(,$attribute:ident)* $(,)?) => {
        #[proc_macro_derive($trait_, attributes($($attribute),*))]
        pub fn $fn_name(input: TokenStream) -> TokenStream {
            let ast = syn::parse(input).unwrap();
            $crate::process($mod_::expand(&ast, stringify!($trait_)))
        }
    }
);

create_derive!(signal, Signal, signal_derive, Value);
