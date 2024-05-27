pub trait Validator {
    type E;

    fn validate(&self) -> Result<(), Self::E>;
}
