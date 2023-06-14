#![allow(dead_code)]

pub use domain::*;

pub mod db;
pub mod domain;

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }
}
