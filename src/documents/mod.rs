mod create;

pub use create::*;

pub trait ExampleDocument {
    fn create_example() -> Self;
}
