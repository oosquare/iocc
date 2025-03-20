mod attrs;
mod impls;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::Result as SynResult;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    match component_impl(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn component_impl(attr: TokenStream, item: TokenStream) -> SynResult<TokenStream2> {
    let attr_data = attrs::parse_attributes(attr)?;
    let expanded = impls::expand_implementation(item, attr_data)?;
    Ok(expanded)
}
