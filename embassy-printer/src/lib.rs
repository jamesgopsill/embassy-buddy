#![no_std]

pub trait Printer {
    type PrinterError;
    type HomeError;

    fn relative_linear_move(
        &self,
        x: f64,
        y: f64,
        z: f64,
    ) -> impl Future<Output = Result<(), Self::PrinterError>>;

    fn home(&self) -> impl Future<Output = Result<(), Self::HomeError>>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        assert!(true)
    }
}
