use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Nothing, Field, Fields, FieldsNamed, FieldsUnnamed, ItemEnum, Result, Variant};

use crate::utils::VecExt;

use super::{Context, PIN};

pub(super) fn parse(cx: &mut Context, mut item: ItemEnum) -> Result<TokenStream> {
    if item.variants.is_empty() {
        return Err(error!(item, "cannot be implemented for enums without variants"));
    }
    let has_field = item.variants.iter().try_fold(false, |has_field, v| {
        if let Some((_, e)) = &v.discriminant {
            Err(error!(e, "cannot be implemented for enums with discriminants"))
        } else if let Fields::Unit = v.fields {
            Ok(has_field)
        } else {
            Ok(true)
        }
    })?;
    if !has_field {
        return Err(error!(item.variants, "cannot be implemented for enums that have no field"));
    }

    let (proj_variants, proj_arms) = variants(cx, &mut item)?;

    let Context { proj_ident, proj_trait, orig_ident, lifetime, .. } = &cx;
    let proj_generics = cx.proj_generics();
    let proj_ty_generics = proj_generics.split_for_impl().1;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut proj_items = quote! {
        enum #proj_ident #proj_generics #where_clause { #(#proj_variants,)* }
    };
    proj_items.extend(quote! {
        impl #impl_generics #proj_trait #ty_generics for ::core::pin::Pin<&mut #orig_ident #ty_generics> #where_clause {
            fn project<#lifetime>(&#lifetime mut self) -> #proj_ident #proj_ty_generics #where_clause {
                unsafe {
                    match self.as_mut().get_unchecked_mut() {
                        #(#proj_arms,)*
                    }
                }
            }
        }
    });

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    Ok(item)
}

fn variants(cx: &mut Context, item: &mut ItemEnum) -> Result<(Vec<TokenStream>, Vec<TokenStream>)> {
    let mut proj_variants = Vec::with_capacity(item.variants.len());
    let mut proj_arms = Vec::with_capacity(item.variants.len());
    for Variant { fields, ident, .. } in &mut item.variants {
        let (proj_pat, proj_body, proj_field) = match fields {
            Fields::Unnamed(fields) => unnamed(cx, fields)?,
            Fields::Named(fields) => named(cx, fields)?,
            Fields::Unit => (TokenStream::new(), TokenStream::new(), TokenStream::new()),
        };
        let Context { orig_ident, proj_ident, .. } = &cx;
        let proj_arm = quote!(#orig_ident::#ident #proj_pat => #proj_ident::#ident #proj_body );
        let proj_variant = quote!(#ident #proj_field);
        proj_arms.push(proj_arm);
        proj_variants.push(proj_variant);
    }

    Ok((proj_variants, proj_arms))
}

fn named(
    cx: &mut Context,
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
) -> Result<(TokenStream, TokenStream, TokenStream)> {
    let mut proj_pat = Vec::with_capacity(fields.len());
    let mut proj_body = Vec::with_capacity(fields.len());
    let mut proj_field = Vec::with_capacity(fields.len());
    for Field { attrs, ident, ty, .. } in fields {
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            cx.push_unpin_bounds(ty.clone());
            let lifetime = &cx.lifetime;
            proj_body.push(quote!(#ident: ::core::pin::Pin::new_unchecked(#ident)));
            proj_field.push(quote!(#ident: ::core::pin::Pin<&#lifetime mut #ty>));
        } else {
            let lifetime = &cx.lifetime;
            proj_body.push(quote!(#ident));
            proj_field.push(quote!(#ident: &#lifetime mut #ty));
        }
        proj_pat.push(ident);
    }

    let proj_pat = quote!({ #(#proj_pat),* });
    let proj_body = quote!({ #(#proj_body),* });
    let proj_field = quote!({ #(#proj_field),* });
    Ok((proj_pat, proj_body, proj_field))
}

fn unnamed(
    cx: &mut Context,
    FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
) -> Result<(TokenStream, TokenStream, TokenStream)> {
    let mut proj_pat = Vec::with_capacity(fields.len());
    let mut proj_body = Vec::with_capacity(fields.len());
    let mut proj_field = Vec::with_capacity(fields.len());
    for (i, Field { attrs, ty, .. }) in fields.iter_mut().enumerate() {
        let x = format_ident!("_x{}", i);
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            cx.push_unpin_bounds(ty.clone());
            let lifetime = &cx.lifetime;
            proj_body.push(quote!(::core::pin::Pin::new_unchecked(#x)));
            proj_field.push(quote!(::core::pin::Pin<&#lifetime mut #ty>));
        } else {
            let lifetime = &cx.lifetime;
            proj_body.push(quote!(#x));
            proj_field.push(quote!(&#lifetime mut #ty));
        }
        proj_pat.push(x);
    }

    let proj_pat = quote!((#(#proj_pat),*));
    let proj_body = quote!((#(#proj_body),*));
    let proj_field = quote!((#(#proj_field),*));
    Ok((proj_pat, proj_body, proj_field))
}