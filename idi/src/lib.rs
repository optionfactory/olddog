use pgx::prelude::*;

pg_module_magic!();

#[pg_extern]
pub fn is_valid_fiscal_code(val: &str) -> bool {
    codice_fiscale::CodiceFiscale::check(val).is_ok()
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::prelude::*;
    use crate::is_valid_fiscal_code;

    #[pg_test]
    fn valid_code_is_valid() {
        assert!(is_valid_fiscal_code("RSSMRA85T10A562S"));
    }

    #[pg_test]
    fn invalid_code_is_invalid() {
        assert_eq!(false, is_valid_fiscal_code("XXXX"));
    }

    #[pg_test]
    fn can_check_is_valid_via_select() {
        assert_eq!(Some(true), Spi::get_one("SELECT is_valid_fiscal_code('RSSMRA85T10A562S')").unwrap());
    }
}

/// This module is required by `cargo pgx test` invocations. 
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}