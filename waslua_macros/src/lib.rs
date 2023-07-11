use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Abi, FnArg, ForeignItem, Item, ItemFn, LitStr, Token,
    Visibility,
};

#[proc_macro_attribute]
pub fn waslua(_attr: TokenStream, input: TokenStream) -> TokenStream {
    std::env::set_var("RUST_BACKTRACE", "1");
    let item = parse_macro_input!(input as Item);
    match item {
        Item::ForeignMod(mut item) => {
            match &mut item.abi.name {
                Some(v) => {
                    if v.value() == "lua".to_string() {
                        *v = LitStr::new("C", v.span());
                    } else {
                        let v = v.span();
                        return quote_spanned!(v => compile_error!("Only \"lua\" abi is allowed."))
                            .into();
                    }
                }
                None => {
                    let v = item.abi.extern_token.span;
                    return quote_spanned!(v => compile_error!("Only \"lua\" abi is allowed."))
                        .into();
                }
            }
            let mut v: Vec<TokenStream2> = Vec::new();
            for x in item.items {
                match x {
                    ForeignItem::Fn(f) => {
                        let f: ItemFn = ItemFn {
                            vis: syn::Visibility::Inherited,
                            block: Box::new({
                                let args = f.sig.inputs.iter().map(|f| match f {
                                    FnArg::Receiver(rec) => {
                                        quote! {#rec}
                                    }
                                    FnArg::Typed(v) => {
                                        let v = &v.pat;
                                        quote!(#v)
                                    }
                                });
                                let fname = &f.sig.ident;
                                let mut f2 = f.clone();
                                f2.attrs = Vec::new();
                                f2.vis = Visibility::Inherited;
                                syn::parse_quote! {{
                                    #[link(wasm_import_module = "lua")]
                                    extern "C" {
                                        #[doc(hidden)] #f2
                                    }
                                    #[allow(unsafe_code)] unsafe { #fname ( #(#args),* ) }
                                }}
                            }),
                            sig: f.sig,
                            attrs: f.attrs,
                        };
                        v.push(quote! {
                            #f
                        })
                    }
                    ForeignItem::Macro(m) => {
                        let m = m.span();
                        return quote_spanned! {
                            m => compile_error!("Macros inside of waslua import blocks are not supported.")
                        }.into();
                    }
                    ForeignItem::Static(v) => {
                        let m = v.span();
                        return quote_spanned! {
                            m => compile_error!("Statics inside of waslua import blocks are not supported.")
                        }.into();
                    }
                    ForeignItem::Type(v) => {
                        let m = v.span();
                        return quote_spanned! {
                            m => compile_error!("Types inside of waslua import blocks are not supported.")
                        }.into();
                    }
                    ForeignItem::Verbatim(ts) => {
                        return quote_spanned!(ts.span() => compile_error!{"Unparseable code"})
                            .into();
                    }
                    _ => {}
                }
            }
            let v = v.iter();
            quote! {#(#v)*}.into()
        }
        Item::Fn(mut item) => {
            item.sig.abi = Some(Abi {
                extern_token: Token![extern](
                    item.clone()
                        .sig
                        .abi
                        .and_then(|a| Some(a.extern_token.span()))
                        .unwrap_or(Span::call_site()),
                ),
                name: Some(LitStr::new(
                    "C",
                    item.sig
                        .abi
                        .and_then(|a| {
                            Some(
                                a.name
                                    .and_then(|v| Some(v.span()))
                                    .unwrap_or(Span::call_site()),
                            )
                        })
                        .unwrap_or(Span::call_site()),
                )),
            });
            item.attrs.push(syn::parse_quote!(#[no_mangle]));
            quote!(#item).into()
        }
        o => {
            let span = o.span();
            quote_spanned! {span => compile_error!{"Expected a extern \"lua\" block or function."}}
                .into()
        }
    }
}
