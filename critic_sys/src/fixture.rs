use proc_macro::TokenStream;

use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input, Error, Ident, ItemFn, ReturnType, Type, Visibility,
};

struct FixtureProducer {
    producer_name: Ident,
    fixture_type: Type,
    producer: ItemFn,
}

impl Parse for FixtureProducer {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let function: ItemFn = input.parse()?;

        // TODO: maybe get rid of this check?
        if !matches!(function.vis, Visibility::Inherited) {
            return Err(
                Error::new_spanned(
                    function.vis,
                    "The visibility of the producer function is irrelevent, only the visibility of the returned fixture matters"
                )
            );
        }

        if !function.sig.inputs.is_empty() {
            return Err(Error::new_spanned(
                function.sig.inputs,
                "Producer functions are not allowed to accept any arguments (currently)",
            ));
        }

        let return_type = match function.sig.output.clone() {
            ReturnType::Default => {
                return Err(Error::new_spanned(
                    function.sig.output,
                    "Producer functions must return a fixture",
                ));
            }
            ReturnType::Type(_, return_type) => return_type,
        };

        Ok(FixtureProducer {
            producer_name: function.sig.ident.clone(),
            fixture_type: *return_type,
            producer: function,
        })
    }
}

pub fn fixture(input: TokenStream) -> TokenStream {
    let producer = parse_macro_input!(input as FixtureProducer);

    let producer_name = producer.producer_name;
    let fixture_type = producer.fixture_type;
    let producer_function = producer.producer;

    let fixture_config_name = Ident::new(
        &format!("__critic_internal_{producer_name}_config"),
        producer_name.span(),
    );

    let runnable_fn = crate::runnable_fn::wrapper(vec![], true, producer_name.clone());

    quote! {
        #[::critic::__internal::linkme::distributed_slice(crate::__critic_test_internals::CRITIC_INTERNAL_FIXTURES)]
        fn #fixture_config_name() -> ::critic::__internal::FixtureConfig {
            #runnable_fn

            critic::__internal::FixtureConfig {
                inputs,
                output: ::std::any::TypeId::of::<#fixture_type>(),
                runnable_fn: &runnable_wrapper,
                module_path: ::std::module_path!().to_string(),
                name: stringify!(#producer_name).to_string(),
            }
        }

        #producer_function
    }.into()
    // quote! {
    //     impl critic::__internal::Fixture for #fixture_type {
    //         fn produce() -> Self {
    //             #producer_name()
    //         }
    //     }
    //
    //     #producer_function
    // }
    // .into()
}
