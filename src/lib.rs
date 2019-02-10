extern crate byteorder;
extern crate hwaddr;
extern crate image;
extern crate log;

pub mod discovery;
pub mod canvas;
pub mod registry;
pub mod device;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
