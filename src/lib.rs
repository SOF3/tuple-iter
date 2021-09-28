//! Generate iterator types for tuples of items implementing the same trait.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Result;

/// Creates an iterator for the shared reference to a tuple as trait objects.
///
/// ```
/// trait Foo {
///     fn bar(&self) -> i32;
/// }
///
/// struct A(i32);
/// impl Foo for A {
///     fn bar(&self) -> i32 { self.0 }
/// }
///
/// struct B(i32);
/// impl Foo for B {
///     fn bar(&self) -> i32 { self.0 }
/// }
///
/// let my_tuple = (A(1), B(2), A(3));
/// let iter = tuple_iter::iter!(my_tuple, (Foo + Send + Sync + 'static; 3));
/// let vec: Vec<i32> = iter.map(|foo| foo.bar()).collect();
/// assert_eq!(vec, vec![1, 2, 3]);
/// ```
#[proc_macro]
pub fn iter(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    iter_impl(false, ts.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Creates an iterator for the mutable reference to a tuple as trait objects.
///
/// ```
/// trait Foo {
///     fn bar(&mut self) -> i32;
/// }
///
/// struct A(i32);
/// impl Foo for A {
///     fn bar(&mut self) -> i32 { self.0 }
/// }
///
/// struct B(i32);
/// impl Foo for B {
///     fn bar(&mut self) -> i32 { self.0 }
/// }
///
/// let mut my_tuple = (A(1), B(2), A(3));
/// let iter = tuple_iter::iter_mut!(my_tuple, (Foo + Send + Sync + 'static; 3));
/// let vec: Vec<i32> = iter.map(|foo| foo.bar()).collect();
/// assert_eq!(vec, vec![1, 2, 3]);
/// ```
#[proc_macro]
pub fn iter_mut(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    iter_impl(true, ts.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn iter_impl(is_mut: bool, ts: TokenStream) -> Result<TokenStream> {
    let (is_mut, ptr) = if is_mut {
        (quote!(mut), quote!(*mut))
    } else {
        (quote!(), quote!(*const))
    };

    let input = syn::parse2::<Input>(ts)?;
    let Input {
        expr,
        _comma,
        _parentheses,
        bounds,
        _semicolon,
        count,
    } = &input;
    let count = *count;

    let ordinal = 0..count;
    let ty_params: Vec<_> = ordinal.clone().map(|i| format_ident!("Ty{}", i)).collect();
    let ordinal = ordinal.map(|i| proc_macro2::Literal::usize_unsuffixed(i).to_token_stream());

    let code = quote! {
        {
            struct __TupleIter<T>(T, usize);

            impl<'t, #(#ty_params),*> Iterator for __TupleIter<&'t #is_mut (#(#ty_params),*)>
                where #(#ty_params: #bounds),* {
                    type Item = &'t #is_mut (dyn #bounds);

                    fn next(&mut self) -> Option<Self::Item> {
                        match self.1 {
                            #(
                                #ordinal => {
                                    self.1 += 1;
                                    let ptr = &#is_mut (self.0).#ordinal as #ptr #ty_params;
                                    let ptr: &#is_mut #ty_params = unsafe { &#is_mut *ptr };
                                    let dyn_ptr: &#is_mut (dyn #bounds) = ptr;
                                    Some(dyn_ptr)
                                },
                            )*
                            _ => None,
                        }
                    }
                }

            __TupleIter(&#is_mut #expr, 0)
        }
    };
    Ok(code)
}

struct Input {
    expr: syn::Expr,
    _comma: syn::Token![,],
    _parentheses: syn::token::Paren,
    bounds: Punctuated<syn::TypeParamBound, syn::Token![+]>,
    _semicolon: syn::Token![;],
    count: usize,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = input.parse::<syn::Expr>()?;
        let comma = input.parse::<syn::Token![,]>()?;
        let inner;
        let parentheses = syn::parenthesized!(inner in input);
        let bounds = Punctuated::parse_separated_nonempty(&inner)?;
        let semicolon = inner.parse::<syn::Token![;]>()?;
        let count = inner.parse::<syn::LitInt>()?;
        let count = count.base10_parse()?;

        Ok(Self {
            expr,
            _comma: comma,
            _parentheses: parentheses,
            bounds,
            _semicolon: semicolon,
            count,
        })
    }
}
