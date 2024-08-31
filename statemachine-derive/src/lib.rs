use proc_macro::TokenStream;
mod statemachine;
use statemachine::statemachine_impl;

#[proc_macro]
pub fn statemachine(item: TokenStream) -> TokenStream {
    statemachine_impl(item)
}
