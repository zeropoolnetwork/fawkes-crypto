extern crate proc_macro;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_str, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Path, Type, Variant
};

#[proc_macro_derive(Signal, attributes(Field, Value))]
pub fn signal_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    expand(&ast, "Signal").into()
}

fn field_idents<'a>(fields: &'a [&'a Field]) -> Vec<&'a Ident> {
    fields
        .iter()
        .map(|f| {
            f.ident
                .as_ref()
                .expect("Tried to get field names of a tuple struct")
        })
        .collect()
}

fn fetch_attr(name: &str, attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
            match meta {
                syn::Meta::NameValue(nv) => {
                    if nv.path.get_ident().map(|i| i.to_string()) == Some(name.to_string()) {
                        match nv.lit {
                            syn::Lit::Str(ref s) => return Some(s.value()),
                            _ => {
                                panic!("attribute {} should be a string", name);
                            }
                        }
                    }
                }
                _ => {
                    panic!("attribute {} should be a string", name);
                }
            }
        }
    }

    None
}

fn unnamed_to_vec(fields: &FieldsUnnamed) -> Vec<&Field> {
    fields.unnamed.iter().collect()
}

fn named_to_vec(fields: &FieldsNamed) -> Vec<&Field> {
    fields.named.iter().collect()
}

fn expand(input: &DeriveInput, _: &str) -> TokenStream {
    let input_type = &input.ident;
    let value_type = parse_str::<Type>(
        &fetch_attr("Value", &input.attrs).expect("attribute value should be defined"),
    )
    .expect("attribute should be a type");

    let field_path = parse_str::<Path>(&fetch_attr("Field", &input.attrs).unwrap_or(String::from("Fr"))).expect("attribute should be a path");

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let body = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => {
                let field_vec = unnamed_to_vec(fields);
                tuple_impl(&field_vec, &field_path)
            }
            Fields::Named(ref fields) => {
                let field_vec = named_to_vec(fields);
                struct_impl(&field_vec, &field_path)
            }
            Fields::Unit => struct_impl(&[], &field_path),
        },
        Data::Enum(ref data_enum) => {
            enum_impl(data_enum.variants.iter().cloned().collect(), &field_path)
        },
        _ => panic!("Only structs can derive a constructor"),
    };

    quote! {
        impl #impl_generics Signal<#field_path> for #input_type#ty_generics #where_clause {
            type Value = #value_type;

            #body

        }
    }
}

fn get_field_types_iter<'a>(fields: &'a [&'a Field]) -> Box<dyn Iterator<Item = &'a Type> + 'a> {
    Box::new(fields.iter().map(|f| &f.ty))
}

fn get_field_types<'a>(fields: &'a [&'a Field]) -> Vec<&'a Type> {
    get_field_types_iter(fields).collect()
}



fn tuple_impl(fields: &[&Field], field_path:&Path) -> TokenStream {
    let var_typenames = get_field_types(&fields);
    let var_ids = (0..fields.len())
        .map(|i| syn::Index::from(i))
        .collect::<Vec<_>>();

    quote! {
        fn get_value(&self) -> Option<Self::Value> {
            Some(Self::Value{#(#var_ids:self.#var_ids.get_value()?),*})
        }

        fn as_const(&self) -> Option<Self::Value> {
            Some(Self::Value{#(#var_ids:self.#var_ids.as_const()?),*})
        }

        fn switch(&self, bit: &CBool<#field_path>, if_else: &Self) -> Self {
            Self( #(self. #var_ids .switch(bit, &if_else. #var_ids)),* )
        }

        fn get_cs(&self) -> &RCS<#field_path> {
            self.0.get_cs()
        }

        fn from_const(cs:&RCS<#field_path>, value: &Self::Value) -> Self {
            Self(#(<#var_typenames>::from_const(cs, &value.#var_ids)),*)
        }

        fn assert_const(&self, value: &Self::Value) {
            #(self. #var_ids .assert_const(&value. #var_ids);)*
        }

        fn inputize(&self) {
            #(self. #var_ids .inputize();)*
        }

        fn assert_eq(&self, other: &Self) {
            #(self. #var_ids .assert_eq(&other. #var_ids);)*
        }

        fn is_eq(&self, other: &Self) -> CBool<#field_path> {
            let mut acc = self.derive_const(&true);
            #(acc &= self. #var_ids .is_eq(&other. #var_ids);)*
            acc
        }

        fn alloc(cs:&RCS<#field_path>, value:Option<&Self::Value>) -> Self {
            Self(#(<#var_typenames>::alloc(cs, value.map(|v| &v.#var_ids))),*)
        }
    }
}

fn struct_impl(fields: &[&Field], field_path:&Path) -> TokenStream {
    let var_names: &Vec<Ident> = &field_idents(fields).iter().map(|f| (**f).clone()).collect();

    let var_name_first = var_names[0].clone();
    let var_typenames = get_field_types(&fields);

    quote! {
        fn get_value(&self) -> Option<Self::Value> {
            Some(Self::Value {#(#var_names: self.#var_names.get_value()?),*})
        }

        fn as_const(&self) -> Option<Self::Value> {
            Some(Self::Value {#(#var_names: self.#var_names.as_const()?),*})
        }

        fn switch(&self, bit: &CBool<#field_path>, if_else: &Self) -> Self {
            Self {#(#var_names: self.#var_names.switch(bit, &if_else.#var_names)),*}
        }

        fn get_cs(&self) -> &RCS<#field_path> {
            self.#var_name_first.get_cs()
        }

        fn from_const(cs:&RCS<#field_path>, value: &Self::Value) -> Self {
            Self {#(#var_names: <#var_typenames>::from_const(cs, &value.#var_names)),*}
        }

        fn assert_const(&self, value: &Self::Value) {
            #(self. #var_names .assert_const(&value. #var_names);)*
        }

        fn inputize(&self) {
            #(self. #var_names .inputize();)*
        }

        fn assert_eq(&self, other: &Self) {
            #(self. #var_names .assert_eq(&other. #var_names);)*
        }

        fn is_eq(&self, other: &Self) -> CBool<#field_path> {
            let mut acc = self.derive_const(&true);
            #(acc &= self. #var_names .is_eq(&other. #var_names);)*
            acc
        }

        fn alloc(cs:&RCS<#field_path>, value:Option<&Self::Value>) -> Self {
            Self {#(#var_names: <#var_typenames>::alloc(cs, value.map(|v| &v.#var_names))),*}
        }


    }
}

// TODO: Implement for tuple variants
fn enum_impl(variants: Vec<Variant>, field_path: &Path) -> TokenStream {
    let variant_names: Vec<Ident> = variants.iter().map(|v| v.ident.clone()).collect();

    let field_names: Vec<Vec<Ident>> = variants
        .iter()
        .map(|v| {
            if v.fields.is_empty() {
                panic!("Variants cannot be empty");
            }

            v.fields
                .iter()
                .map(|f| f.ident.clone().unwrap())
                .collect()
        })
        .collect();

    // for destructuring the second argument
    let other_field_names: Vec<Vec<Ident>> = field_names.iter().map(|fields| {
        fields.iter().map(|field: &Ident| {
            Ident::new(&format!("__other_{}", field), field.span())
        }).collect::<Vec<Ident>>()
    }).collect();

    let first_field_names = field_names.iter().map(|names| names[0].clone());

    quote! {
        fn get_value(&self) -> Option<Self::Value> {
            match self {
                #(Self::#variant_names { #(#field_names),* } => {
                    Some(Self::Value::#variant_names {#(#field_names: #field_names.get_value()?),*})
                }),*
            }
        }

        fn as_const(&self) -> Option<Self::Value> {
            match self {
                #(Self::#variant_names { #(#field_names),* } => {
                    Some(Self::Value::#variant_names {#(#field_names: #field_names.as_const()?),*})
                }),*
            }
        }

        fn switch(&self, bit: &CBool<#field_path>, if_else: &Self) -> Self {
            match self {
                #(Self::#variant_names { #(#field_names),* } => {
                    if let Self::#variant_names { #(#field_names: #other_field_names),* } = if_else {
                        Self::#variant_names {#(#field_names: #field_names.switch(bit, #other_field_names)),*}
                    } else {
                        panic!("Signal::switch: variants do not match");
                    }
                }),*
            }
        }

        fn get_cs(&self) -> &RCS<#field_path> {
            match self {
                #(Self::#variant_names { #(#field_names),* } => {
                    #first_field_names.get_cs()
                }),*
            }
        }

        fn from_const(cs:&RCS<#field_path>, value: &Self::Value) -> Self {
            match value {
                #(Self::Value::#variant_names { #(#field_names),* } => {
                    Self::#variant_names {#(#field_names: Signal::from_const(cs, #field_names)),*}
                }),*
            }
        }

        fn assert_const(&self, value: &Self::Value) {
            match self {
                #(Self::#variant_names { #(#field_names),* } => {
                    if let Self::Value::#variant_names { #(#field_names: #other_field_names),* } = value {
                        #(#field_names.assert_const(#other_field_names);)*
                    } else {
                        panic!("Signal::assert_const: variants do not match");
                    }
                }),*
            }
        }

        fn inputize(&self) {
            match self {
                #(Self::#variant_names { #(#field_names),* } => {
                    #(#field_names.inputize();)*
                }),*
            }
        }

        fn assert_eq(&self, other: &Self) {
            match self {
                #(Self::#variant_names { #(#field_names),* } => {
                    if let Self::#variant_names { #(#field_names: #other_field_names),* } = other {
                        #(#field_names.assert_eq(#other_field_names);)*
                    } else {
                        panic!("Signal::assert_eq: variants do not match");
                    }
                }),*
            }
        }

        fn is_eq(&self, other: &Self) -> CBool<#field_path> {
            let mut acc = self.derive_const(&true);

            match self {
                #(Self::#variant_names { #(#field_names),* } => {
                    if let Self::#variant_names { #(#field_names: #other_field_names),* } = other {
                        #(acc &= #field_names.is_eq(#other_field_names);)*
                    }
                }),*
            }

            acc
        }

        fn alloc(cs: &RCS<#field_path>, value: Option<&Self::Value>) -> Self {
            if let Some(value) = value {
                match value {
                    #(Self::Value::#variant_names { #(#field_names),* } => {
                        Self::#variant_names { #(#field_names: Signal::alloc(cs, Some(#field_names))),* }
                    }),*
                }
            } else {
                panic!("Signal::alloc on enums requires value to not be empty");
            }
        }
    }
}
