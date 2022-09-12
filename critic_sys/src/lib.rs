use proc_macro::TokenStream;

mod fixture;
mod runnable_fn;
mod test;

#[proc_macro_attribute]
pub fn fixture(_attr: TokenStream, input: TokenStream) -> TokenStream {
    fixture::fixture(input)
}

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    test::test(input)
}
