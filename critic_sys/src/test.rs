use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, FnArg, Ident, ItemFn};

struct TestFunction {
    body: ItemFn,
}

impl Parse for TestFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let body: ItemFn = input.parse()?;

        // TODO: checks

        Ok(TestFunction { body })
    }
}

pub fn test(input: TokenStream) -> TokenStream {
    let test_function = parse_macro_input!(input as TestFunction);
    let test_body = test_function.body;
    let test_name = test_body.sig.ident.clone();
    let test_config_name = Ident::new(
        &format!("__critic_internal_{test_name}_config"),
        test_name.span(),
    );

    let arg = test_body
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(arg) => arg.clone(),
            FnArg::Receiver(_) => panic!("tests are not allowed to take in self"),
        })
        .collect::<Vec<_>>();

    let basic_runnable = crate::runnable_fn::wrapper(arg, false, test_name.clone());

    quote!(
        #[critic::__internal::linkme::distributed_slice(crate::__critic_test_internals::CRITIC_INTERNAL_TESTS)]
        fn #test_config_name() -> critic::__internal::TestConfig {
            #basic_runnable

            critic::__internal::TestConfig {
                should_panic: false,
                subprocess: false,
                inputs,
                exclusion_group: None,
                runnable_fn: &runnable_wrapper,
                module_path: ::std::module_path!().to_string(),
                name: stringify!(#test_name).to_string(),
            }
        }

        #test_body
    )
    .into()
}
