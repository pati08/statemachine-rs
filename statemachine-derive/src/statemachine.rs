use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, FieldsNamed, Ident, Token};

struct Link {
    from: Ident,
    to: Ident,
    method: Ident,
}

impl Link {
    fn to_tokens(
        &self,
        tokens: &mut proc_macro2::TokenStream,
        machine_name: Ident,
        fields: Option<FieldsNamed>,
    ) {
        let from = self.from.clone();
        let to = self.to.clone();
        let method = self.method.clone();
        let fields = fields.map(|f| f.named).unwrap_or(Punctuated::new());
        let fields = fields
            .into_iter()
            .map(|f| f.ident.expect("Expected field name"));
        tokens.extend(quote!(
            impl #machine_name<#from> {
                fn #method(self) -> #machine_name<#to> {
                    #machine_name {
                        _phantom_data: std::marker::PhantomData,
                        #(
                            #fields: self.#fields
                        )*
                    }
                }
            }
        ))
    }
}

struct ParsedMachine {
    is_public: bool,
    name: Ident,
    states: Vec<Ident>,
    links: Vec<Link>,
    fields: Option<FieldsNamed>,
}

pub fn statemachine_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ParsedMachine);
    let states = input.states;
    let links = input.links;
    let is_public = input.is_public;
    let name = input.name;

    let mut tokens = proc_macro2::TokenStream::new();

    if is_public {
        tokens.extend(quote!(pub));
    }

    let fields = input
        .fields
        .clone()
        .map(|f| f.named)
        .unwrap_or(Punctuated::new());
    // fields.push(syn::Field {
    //     attrs: Vec::new(),
    //     vis: syn::Visibility::Inherited,
    //     mutability: syn::FieldMutability::None,
    //     ident: Some(Ident::new("_phantom_data", proc_macro2::Span::call_site())),
    //     colon_token: Some(Token![;]),
    //     ty: syn::Type::Path(syn::TypePath {
    //         qself: None,
    //         path: syn::Path
    //     })
    // });

    let body = quote!(
        struct #name<T> {
            _phantom_data: std::marker::PhantomData<T>,
            #fields
        }
        #(
            struct #states;
        )*
    );

    tokens.extend(body);

    for link in links {
        link.to_tokens(&mut tokens, name.clone(), input.fields.clone());
    }

    tokens.into()
}

impl Parse for ParsedMachine {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut states = Vec::<Ident>::new();
        let mut links = Vec::<Link>::new();

        if input.is_empty() {
            panic!("At least one state must be defined.");
        }

        let is_public = input.peek(Token![pub]);
        if is_public {
            input.parse::<Token![pub]>()?;
        }
        let name = input.parse::<syn::Ident>()?;

        let fields = input.parse::<FieldsNamed>().ok();

        input.parse::<Token![;]>()?;

        while !input.is_empty() {
            let from = input.parse::<syn::Ident>()?;
            input.parse::<Token![->]>()?;
            let to = input.parse::<syn::Ident>()?;
            input.parse::<Token![:]>()?;
            let method = input.parse::<syn::Ident>()?;

            if !states.contains(&from) {
                states.push(from.clone());
            }
            if !states.contains(&to) {
                states.push(to.clone());
            }

            links.push(Link { from, to, method });

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ParsedMachine {
            states,
            links,
            is_public,
            name,
            fields,
        })
    }
}
