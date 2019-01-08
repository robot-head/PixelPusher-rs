extern crate byteorder;
extern crate hwaddr;
extern crate timer;
extern crate chrono;

pub mod discovery;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
