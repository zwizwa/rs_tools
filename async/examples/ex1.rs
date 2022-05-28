#![allow(unused_imports)]
extern crate rst_async;
use futures::Future;
use rst_async::test;
pub fn main() {
    let fut = test::test123().await;
    println!("hello {}", fut)
}
