mod create;

pub use create::Create;

pub trait ExampleDocument {
    fn create_example() -> Self;
}
