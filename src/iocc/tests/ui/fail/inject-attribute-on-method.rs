use iocc::prelude::*;

pub struct Test;

#[component]
impl Test {
    #[inject]
    pub fn method(self) -> Self {
        todo!()
    }
}

fn main() {}
