extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MutView)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        panic!("Macro must be applied to struct")
    };

    let new_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let ty_string = ty.to_token_stream().to_string();

        //TODO: Match using regex
        let converted_type = |ty_string: &str| {
            if ty_string.starts_with("ByteView<") {
                return quote! {let (#name, buf) = ByteView::mut_view(buf)};
            } else if ty_string.starts_with("MulByteView<") {
                return quote! {let (#name, buf) = MulByteView::mut_view(buf)};
            } else if ty_string.starts_with("ArrayView<") {
                return quote! {let (#name, buf) = ArrayView::mut_view(buf)};
            } else if ty_string.starts_with("VarArrayView<") {
                return quote! {let (#name, buf) = VarArrayView::mut_view(buf, buf.len())};
            } else if ty_string.starts_with("Option") {
                return quote! {let #name = <#ty as ::core::default::Default>::default()};
            } else {
                return quote! {let (#name, buf) = <#ty>::mut_view(buf)};
            }
        };

        converted_type(&ty_string)
    });

    let field_names = fields.iter().map(|f| {
        let name = &f.ident;

        quote! {#name}
    });

    let expanded = quote! {
        impl<'a> #struct_name<'a> {
            pub fn mut_view(buf: &'a mut [u8]) -> (Self, &'a mut [u8]){
                    #(#new_fields;)*

                    (Self {
                       #(#field_names,)*
                    }, buf)
            }
        }
    };

    expanded.into()
}
