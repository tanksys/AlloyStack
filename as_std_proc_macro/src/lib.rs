use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use syn::{
    __private::{quote::__private::ext::RepToTokensExt, ToTokens},
    parse2,
};

#[proc_macro_derive(FaasData)]
pub fn derive_verify(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // derive_verify_impl(item.into()).into()
    derive_verify_impl(item.into()).into()
}

fn derive_verify_impl(t: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let input_struct: syn::DeriveInput = match parse2(t) {
        Ok(it) => it,
        Err(e) => return e.into_compile_error(),
    };
    let struct_name = &input_struct.ident;
    // println!("{:?}", struct_name);

    let data_struct = match input_struct.data {
        syn::Data::Struct(it) => it,
        _ => panic!("do not use Verify trait with enum or union."),
    };

    let mut tys = vec![];
    if let syn::Fields::Named(fields) = data_struct.fields {
        for field in fields.named.iter() {
            let tokens = field.ty.to_token_stream();
            let token = tokens.next().unwrap();
            // println!("{:?}", token.to_string());
            tys.push(token.to_string())
        }
    };

    let mut hasher = DefaultHasher::new();
    tys.join("").hash(&mut hasher);
    let fingerprint: u64 = hasher.finish();

    quote::quote! {
        impl as_hostcall::Verify for #struct_name {
            fn __fingerprint() -> u64 {
                #fingerprint
            }
        }
    }
}

#[test]
fn faas_arg_impl_test() {
    assert_eq!(
        derive_verify_impl("struct my_args {a: i64}".parse().unwrap()).to_string(),
        "impl as_hostcall :: Verify for my_args { fn __fingerprint () -> u64 { 13418197873283780248u64 } }"
    );

    assert_eq!(
        derive_verify_impl("struct my_args {a: String, b:i32,}".parse().unwrap()).to_string(),
        "impl as_hostcall :: Verify for my_args { fn __fingerprint () -> u64 { 7169761655847290832u64 } }"
    );
}
