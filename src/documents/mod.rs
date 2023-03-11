mod create;
mod records;

pub use create::*;
pub use records::*;

pub trait ExampleDocument {
    fn create_example() -> Self;
}
