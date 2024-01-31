use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics};

#[proc_macro_derive(VertAttrBuilder)]
pub fn derive_vert_attrs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);

    //let attrs = add_repr_c(input.attrs);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fill_attrs = generate_attrs(&input.data);

    let expanded = quote! {
        // The generated impl
        impl #impl_generics VertAttrBuilder for #name #ty_generics #where_clause {
            fn vert_attrs() -> citro3d::attrib::Info {
                let mut attrs = citro3d::attrib::Info::new();
                #fill_attrs
                attrs
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::bevy_3ds_render::vertattr::VertAttrs));
        }
    }
    generics
}

/*fn add_repr_c(mut attrs: Vec<Attribute>) -> Vec<Attribute> {
    attrs.push(Attribute {
        pound_token: Token![#](Span::call_site()),
        style: Outer,
        bracket_token: Bracket(Span::call_site()),
        meta: syn::Meta::List(MetaList {
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter([PathSegment {
                    ident: Ident::new("repr", Span::call_site()),
                    arguments: syn::PathArguments::None,
                }]),
            },
            delimiter: syn::MacroDelimiter::Paren(Paren(Span::call_site())),
            tokens: "C".parse().unwrap(),
        }),
    });
    attrs
}*/

fn generate_attrs(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().enumerate().map(|(idx, f)| {
                    let ty = &f.ty;
                    let reg_name = format!("reg{}", idx).parse::<TokenStream>().unwrap();
                    quote_spanned! {f.span()=>
                        let #reg_name = citro3d::attrib::Register::new(#idx as u16).unwrap();
                        attrs.add_loader(#reg_name, #ty::FORMAT, #ty::SIZE).unwrap();
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unnamed(ref fields) => {
                let recurse = fields.unnamed.iter().enumerate().map(|(idx, f)| {
                    let ty = &f.ty;
                    let reg_name = format!("reg{}", idx).parse::<TokenStream>().unwrap();
                    quote_spanned! {f.span()=>
                        let #reg_name = citro3d::attrib::Register::new(#idx as u16).unwrap();
                        attrs.add_loader(#reg_name, #ty::FORMAT, #ty::SIZE).unwrap();
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
