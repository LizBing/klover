/*
 * Copyright 2025 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use proc_macro::{Literal, TokenStream};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse::Parse, Type, Expr, LitStr};
use syn::{parse_macro_input, Ident, Token};

mod kw {
    syn::custom_keyword!(product);
}

enum Keyword {
    Product(kw::product),
}

struct VMFlag {
    name: Ident,
    ty: Type,
    default: Expr,
    cons: Expr,
    desc: LitStr
}

impl Parse for VMFlag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::product) {
            Keyword::Product(input.parse()?);
        } else if !input.is_empty() {
            return Err(input.error("Should be product."));
        }

        let content;
        syn::parenthesized!(content in input);

        let name = content.parse()?;
        content.parse::<Token![,]>()?;
        let ty  = content.parse()?;
        content.parse::<Token![,]>()?;
        let default  = content.parse()?;
        content.parse::<Token![,]>()?;
        let cons  = content.parse()?;
        content.parse::<Token![,]>()?;
        let desc  = content.parse()?;

        Ok(VMFlag { name: name, ty: ty, default: default, cons: cons, desc: desc })
    }
}

struct VMFlags {
    name: Ident,
    flags: Vec<VMFlag>
}

impl Parse for VMFlags {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let mut flags = Vec::new();

        let content;
        syn::braced!(content in input);

        while !content.is_empty() {
            flags.push(content.parse()?);
            if content.peek(Token![;]) {
                content.parse::<Token![;]>()?;
            }
        }

        Ok(VMFlags { name: name, flags: flags })        
    }
}

fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[proc_macro]
pub fn def_vm_flags(input: TokenStream) -> TokenStream {
    let imports = quote! {
        use crate::flag_sys::vm_flag::{VMFlag, VMFlagData};
    };

    let mut flag_defs = Vec::new();
    let mut flag_map_entries = Vec::new();
    let VMFlags { name, flags } = parse_macro_input!(input as VMFlags);

    for n in &flags {
        let VMFlag { name, ty, default, cons, desc } = n;

        let name_str = name.to_string();
        let name_lit = name_str.as_str();

        let enum_member_ident = {
            let mut buf = capitalize_first_letter(ty.to_token_stream().to_string().as_str());
            buf.push_str("Flag");
            
            Ident::new(&buf, Span::call_site())
        };

        flag_defs.push(quote! {
            #[doc = #desc]
            pub static #name: VMFlagData<#ty> = VMFlagData::new(
                #name_lit,
                #default,
                #cons,
                #desc
            );
        });

        flag_map_entries.push(quote! {
            #name_lit => VMFlag::#enum_member_ident(&#name),
        });
    }

    let expanded = quote! {
        #imports

        #(#flag_defs)*

        pub static #name: phf::Map<&str, VMFlag> = phf::phf_map! {
            #(#flag_map_entries)*
        };
    };

    TokenStream::from(expanded)
}
