//! Defines a builder for the given struct
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return quote_spanned!(name.span() =>
                            compile_error!("Can only define builders for structs");)
            .into()
        }
    };
    let ret = builder_from_struct(name, data);
    ret
}

#[proc_macro_derive(Consumer)]
pub fn consumer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return quote_spanned!(name.span() =>
                            compile_error!("Can only define consumers for structs");)
            .into()
        }
    };
    let ret = consumer_from_struct(name, data);
    ret
}

fn consumer_from_struct(id: &syn::Ident, item: &syn::DataStruct) -> TokenStream {
    let mut field_names: Vec<&syn::Ident> = vec![];
    let mut field_types: Vec<&syn::Type> = vec![];

    for field in item.fields.iter() {
        match &field.ident {
            Some(id) => field_names.push(id),
            None => {
                return quote_spanned!(field.span() => compile_error!("Expected identifier");)
                    .into()
            }
        };
        field_types.push(&field.ty);
    }

    let struct_fields: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .zip(field_types.iter())
        .map(|(name, ty)| {
            quote!(
            #name : Option<#ty>
            )
        })
        .collect();

    let struct_generics: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|name| {
            let name = format_ident!("{name}SET");
            quote!(
            const #name : bool
            )
        })
        .collect();

    let name = format_ident!("{id}Consumer");

    let impls = consumer_functions(id, field_names, field_types);

    let builder_struct = quote!(
        pub struct #name<#(#struct_generics),*> {
            #(#struct_fields),*
        }
    );
    quote!(

        #builder_struct

        #impls
    )
    .into()
}

/// Generates a set of implementations
fn consumer_functions(
    id: &syn::Ident,
    field_names: Vec<&proc_macro2::Ident>,
    field_types: Vec<&syn::Type>,
) -> proc_macro2::TokenStream {
    let struct_name = format_ident!("{id}Consumer");
    let struct_name = quote!(#struct_name);
    let assign_statements: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|name| {
            quote!(
                #name : self.#name
            )
            .into()
        })
        .collect();
    let ret: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .zip(field_types)
        .map(|(name, ty)| {
            let mut struct_generics: Vec<proc_macro2::TokenStream> = vec![];
            for inner_name in field_names.iter() {
                if !name.eq(inner_name) {
                    let inner_name = format_ident!("{inner_name}CONSUMED");
                    struct_generics.push(quote!(
                        const #inner_name : bool
                    ));
                }
            }
            let requirement: Vec<proc_macro2::TokenStream> = field_names
                .iter()
                .map(|inner_name| {
                    if !name.eq(inner_name) {
                        let inner_name = format_ident!("{inner_name}CONSUMED");
                        quote!(
                            #inner_name
                        )
                    } else {
                        quote!(false)
                    }
                })
                .collect();
            let res_requirement: Vec<proc_macro2::TokenStream> = field_names
                .iter()
                .map(|inner_name| {
                    if !name.eq(inner_name) {
                        let inner_name = format_ident!("{inner_name}CONSUMED");
                        quote!(
                            #inner_name
                        )
                    } else {
                        quote!(true)
                    }
                })
                .collect();
            let fn_name = format_ident!("consume_{name}");
            let fn_name = quote!(#fn_name);
            quote!(
                impl<#(#struct_generics),*> #struct_name <#(#requirement),*>{
                    pub fn #fn_name(mut self) -> (#ty,#struct_name<#(#res_requirement),*>){
                        let ret = self.#name.unwrap();
                        self.#name = None;
                        (ret,
                        #struct_name {
                            #(#assign_statements),*
                        })
                    }
                }
            )
        })
        .collect();
    let mut struct_generics: Vec<proc_macro2::TokenStream> = vec![];
    for inner_name in field_names.iter() {
        let inner_name = format_ident!("{inner_name}CONSUMED");
        struct_generics.push(quote!(
            const #inner_name : bool

        ));
    }
    let complete_requirement: Vec<proc_macro2::TokenStream> =
        field_names.iter().map(|_inner_name| quote!(true)).collect();
    let after_new: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|_inner_name| quote!(false))
        .collect();

    let assign_statements: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|name| {
            quote!(
                #name : Some(self.#name)
            )
            .into()
        })
        .collect();
    let new_builder = quote!(
        impl #id {
            pub fn consumer(self) -> #struct_name<#(#after_new),*>{
                #struct_name{
                    #(#assign_statements),*
                }
            }
        }
    );
    let complete_builder = quote!(
        impl #struct_name<#(#complete_requirement),*> {
            pub fn consume(self) {
            }
        }
    );
    quote!(
        #(#ret)*
        // Defines new function and completion function
        #new_builder
        #complete_builder
    )
}

fn builder_from_struct(id: &syn::Ident, item: &syn::DataStruct) -> TokenStream {
    let mut field_names: Vec<&syn::Ident> = vec![];
    let mut field_types: Vec<&syn::Type> = vec![];

    for field in item.fields.iter() {
        match &field.ident {
            Some(id) => field_names.push(id),
            None => {
                return quote_spanned!(field.span() => compile_error!("Expected identifier,cool_error");)
                    .into()
            }
        };
        field_types.push(&field.ty);
    }

    let struct_fields: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .zip(field_types.iter())
        .map(|(name, ty)| {
            quote!(
            #name : Option<#ty>
            )
        })
        .collect();

    let struct_generics: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|name| {
            let name = format_ident!("{name}SET");
            quote!(
            const #name : bool
            )
        })
        .collect();

    let name = format_ident!("{id}Builder");

    let impls = builder_functions(id, field_names, field_types);

    let builder_struct = quote!(
        pub struct #name<#(#struct_generics),*> {
            #(#struct_fields),*
        }
    );
    quote!(

        #builder_struct

        #impls
    )
    .into()
}

/// Generates a set of implementations
fn builder_functions(
    id: &syn::Ident,
    field_names: Vec<&proc_macro2::Ident>,
    field_types: Vec<&syn::Type>,
) -> proc_macro2::TokenStream {
    let struct_name = format_ident!("{id}Builder");
    let struct_name = quote!(#struct_name);
    let assign_statements: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|name| {
            quote!(
                #name : self.#name
            )
            .into()
        })
        .collect();
    let ret: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .zip(field_types)
        .map(|(name, ty)| {
            let mut struct_generics: Vec<proc_macro2::TokenStream> = vec![];
            for inner_name in field_names.iter() {
                if !name.eq(inner_name) {
                    let inner_name = format_ident!("{inner_name}SET");
                    struct_generics.push(quote!(
                        const #inner_name : bool
                    ));
                }
            }
            let requirement: Vec<proc_macro2::TokenStream> = field_names
                .iter()
                .map(|inner_name| {
                    if !name.eq(inner_name) {
                        let inner_name = format_ident!("{inner_name}SET");
                        quote!(
                            #inner_name
                        )
                    } else {
                        quote!(false)
                    }
                })
                .collect();
            let res_requirement: Vec<proc_macro2::TokenStream> = field_names
                .iter()
                .map(|inner_name| {
                    if !name.eq(inner_name) {
                        let inner_name = format_ident!("{inner_name}SET");
                        quote!(
                            #inner_name
                        )
                    } else {
                        quote!(true)
                    }
                })
                .collect();
            let fn_name = format_ident!("set_{name}");
            let fn_name = quote!(#fn_name);
            quote!(
                impl<#(#struct_generics),*> #struct_name <#(#requirement),*>{
                    pub fn #fn_name(mut self, #name:#ty) -> #struct_name<#(#res_requirement),*>{
                        self.#name = Some(#name);
                        #struct_name {
                            #(#assign_statements),*
                        }
                    }
                }
            )
        })
        .collect();
    let mut struct_generics: Vec<proc_macro2::TokenStream> = vec![];
    for inner_name in field_names.iter() {
        let inner_name = format_ident!("{inner_name}SET");
        struct_generics.push(quote!(
            const #inner_name : bool

        ));
    }
    let complete_requirement: Vec<proc_macro2::TokenStream> =
        field_names.iter().map(|_inner_name| quote!(true)).collect();
    let after_new: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|_inner_name| quote!(false))
        .collect();

    let assign_statements: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|name| {
            quote!(
                #name : None
            )
            .into()
        })
        .collect();
    let complete_statements: Vec<proc_macro2::TokenStream> = field_names
        .iter()
        .map(|name| {
            quote!(
                // Note unwrap here is safe as the flags ensure that there is no iusse
                #name : self.#name.unwrap()
            )
            .into()
        })
        .collect();
    let new_builder = quote!(
        impl #struct_name < #(#after_new),* > {
            pub fn new() -> #struct_name<#(#after_new),*>{
                #struct_name{
                    #(#assign_statements),*
                }
            }
        }
        impl #id {
            pub fn builder() -> #struct_name<#(#after_new),*>{
                #struct_name{
                    #(#assign_statements),*
                }
            }
        }
    );
    let complete_builder = quote!(
        impl #struct_name<#(#complete_requirement),*> {
            pub fn complete(self) -> #id {
                #id {
                    #(#complete_statements),*
                }
            }
        }
    );
    quote!(
        #(#ret)*
        // Defines new function and completion function
        #new_builder
        #complete_builder
    )
}
