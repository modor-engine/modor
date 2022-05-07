#[macro_export]
macro_rules! no_mutation {
    ($($tokens:tt)*) => { $($tokens)* };
}

#[cfg(test)]
mod coverage_tests {
    #[test]
    fn disable_mutations() {
        assert_eq!(no_mutation!(10 + 20), 30);
    }
}
