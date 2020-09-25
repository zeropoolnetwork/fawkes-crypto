extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse::Error as ParseError;

mod signal;

trait Output {
    fn process(self) -> TokenStream;
}

impl Output for proc_macro2::TokenStream {
    fn process(self) -> TokenStream {
        self.into()
    }
}

impl Output for Result<proc_macro2::TokenStream, ParseError> {
    fn process(self) -> TokenStream {
        match self {
            Ok(ts) => ts.into(),
            Err(e) => e.to_compile_error().into(),
        }
    }
}

macro_rules! create_derive(
    ($mod_:ident, $trait_:ident, $fn_name: ident $(,$attribute:ident)* $(,)?) => {
        #[proc_macro_derive($trait_, attributes($($attribute),*))]
        pub fn $fn_name(input: TokenStream) -> TokenStream {
            let ast = syn::parse(input).unwrap();
            Output::process($mod_::expand(&ast, stringify!($trait_)))
        }
    }
);

create_derive!(signal, Signal, signal_derive, Value);
