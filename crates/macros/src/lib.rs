use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Agents)]
pub fn agents_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_agents_macro(&ast)
}

fn impl_agents_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let mut call_tokens = quote!();

    for field in fields {
        let field_name = field.ident.clone();

        if field_name.is_some() {
            call_tokens.extend(quote!(
                self.#field_name.update(env, rng);
            ));
        }
    }

    let output = quote! {
        impl AgentSet for #name {
            fn update(&mut self, env: &mut Env, rng: &mut Rng) {
                #call_tokens
            }
        }
    };

    TokenStream::from(output)
}
