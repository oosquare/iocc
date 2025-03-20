use std::convert::Infallible;

use iocc::prelude::*;

pub struct Test1;

#[component]
impl Test1 {
    #[inject]
    pub fn new() -> Self {
        todo!()
    }
}

pub struct Test2;

#[component]
impl Test2 {
    #[inject]
    pub fn new() -> Test2 {
        todo!()
    }
}

pub struct Test3;

#[component]
impl Test3 {
    #[inject]
    pub fn new() -> Result<Self, Infallible> {
        todo!()
    }
}

pub struct Test4;

#[component]
impl Test4 {
    #[inject]
    pub fn new() -> Result<Test4, Infallible> {
        todo!()
    }
}

pub struct Test5;

#[component]
impl Test5 {
    #[inject]
    pub fn new() -> std::result::Result<Test5, Infallible> {
        todo!()
    }
}

fn main() {}
