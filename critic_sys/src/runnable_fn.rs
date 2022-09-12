use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, PatType, Type};

pub fn wrapper(inputs: Vec<PatType>, output: bool, test_fn_name: Ident) -> TokenStream {
    let ((safe_input_names, _input_names), input_types): ((Vec<_>, Vec<_>), Vec<_>) = inputs
        .iter()
        .enumerate()
        .map(|(i, pat)| {
            (
                (
                    Ident::new(&format!("input{i}"), Span::call_site()),
                    pat.pat.clone(),
                ),
                pat.ty.clone(),
            )
        })
        .unzip();

    let test_fn_run = quote! {
        #test_fn_name ( #(#safe_input_names.guard_extract()),* )
    };

    let run_fn = if output {
        quote! {
            let output = #test_fn_run;

            runnable_input.receiver.receive_output(output);
        }
    } else {
        quote! {
            let output: () = #test_fn_run;
        }
    };

    let inner_type: Vec<_> = input_types
        .clone()
        .into_iter()
        .map(|ty| match ty.as_ref() {
            Type::Reference(r) => r.elem.clone(),
            _ => ty,
        })
        .collect();

    let (input_ref_variant, extractor_name): (Vec<_>, Vec<_>) = input_types
        .into_iter()
        .map(|ty| match *ty {
            Type::Reference(reference) => {
                if reference.lifetime.is_some() {
                    panic!("lifetimes are unsupported on inputs references");
                }

                if reference.mutability.is_some() {
                    (quote! { Exclusive }, quote! { exclusive })
                } else {
                    (quote! { Shared }, quote! { shared })
                }
            }
            _ => (quote! { Owned }, quote! { owned }),
        })
        .unzip();

    quote! {
        fn runnable_wrapper(
            mut runnable_input: ::critic::__internal::RunnableInput,
        ) -> ::std::result::Result<(), Box<dyn ::std::error::Error>> {
            #( let mut #safe_input_names = runnable_input.dependencies.#extractor_name (::std::any::TypeId::of::<#inner_type>());)*

            #run_fn

            Ok(())
        }

        let inputs = vec![
            #(::critic::__internal::InputRef::#input_ref_variant (::std::any::TypeId::of::<#inner_type>())),*
        ];
    }
}
