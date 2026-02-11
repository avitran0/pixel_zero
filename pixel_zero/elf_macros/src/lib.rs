use proc_macro::TokenStream;
use quote::quote;
use syn::{
    LitInt, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

#[repr(C)]
struct MetaHeader {
    magic: [u8; 8],
    version: u32,
    name_len: u32,
}

impl MetaHeader {
    const MAGIC: [u8; 8] = *b"gamemeta";
}

#[proc_macro]
pub fn embed_metadata(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input with MetaArgs::parse);

    let name = args.name.value();
    let version = args.version;

    let mut blob = Vec::with_capacity(size_of::<MetaHeader>());

    blob.extend_from_slice(&MetaHeader::MAGIC);
    blob.extend_from_slice(&version.to_le_bytes());
    blob.extend_from_slice(&(name.len() as u32).to_le_bytes());
    blob.extend_from_slice(name.as_bytes());

    let length = blob.len();

    let byte_tokens = blob.iter().map(|b| quote! {#b});
    let expanded = quote! {
        #[used]
        #[unsafe(link_section = ".gamemeta")]
        #[unsafe(no_mangle)]
        static METADATA: [u8; #length] = [#(#byte_tokens),*];
    };

    TokenStream::from(expanded)
}

struct MetaArgs {
    name: LitStr,
    version: u32,
}

impl Parse for MetaArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut version = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            if ident == "name" {
                let lit: LitStr = input.parse()?;
                name = Some(lit);
            } else if ident == "version" {
                let lit: LitInt = input.parse()?;
                version = Some(lit.base10_parse::<u32>()?);
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "expected `name` or `version`",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(MetaArgs {
            name: name.ok_or_else(|| syn::Error::new(input.span(), "missing `name`"))?,
            version: version.ok_or_else(|| syn::Error::new(input.span(), "missing `version`"))?,
        })
    }
}
