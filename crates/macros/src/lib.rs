use proc_macro::TokenStream;
use quote::quote;

extern crate self as bourse_de;

/// Agent iteration macro
///
/// Implements the `AgentSet` trait for a struct
/// with fields of agent types. It's often the case
/// we want to implement `update` function that
/// iterates over a heterogeneous set of agents,
/// which this macro automates. For example
///
/// ```no_rust
/// #[derive(Agents)]
/// struct SimAgents {
///     a: AgentTypeA,
///     b: AgentTypeB,
/// }
/// ```
///
/// expands to
///
/// ```no_rust
/// struct SimAgents {
///     a: AgentTypeA,
///     b: AgentTypeB,
/// }
///
/// impl AgentSet for SimAgents {
///     fn update<R: RngCore>(
///         &mut self, env: &mut Env, rng: &mut R
///     ) {
///         self.a.update(env, rng);
///         self.b.update(env, rng);
///     }
/// }
/// ```
///
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
        impl bourse_de::agents::AgentSet for #name {
            fn update<R: rand::RngCore>(&mut self, env: &mut bourse_de::Env, rng: &mut R) {
                #call_tokens
            }
        }
    };

    TokenStream::from(output)
}
