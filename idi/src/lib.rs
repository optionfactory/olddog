use std::ffi::CStr;
use std::io::Write;
use pgx::prelude::*;
use pgx::StringInfo;
use serde::{Serialize, Deserialize};

pg_module_magic!();

#[pg_extern]
fn is_valid_fiscal_code(val: &str) -> bool {
    codice_fiscale::CodiceFiscale::check(val).is_ok()
}

#[pg_extern]
fn emojify(code: &str) -> &'static str {
    emojis::get_by_shortcode(code).expect("Invalid code").as_str()
}

#[pg_extern]
fn list_emojis() -> SetOfIterator<'static, &'static str> {
    SetOfIterator::new(emojis::iter().map(|e| e.as_str()))
}

#[derive(Serialize, Deserialize, PostgresType)]
#[inoutfuncs]
pub struct Url {
    scheme: String,
    host: String,
}

impl InOutFuncs for Url {
    fn input(input: &CStr) -> Self where Self: Sized {
        let mut parts = input.to_str().expect("Failed to convert to str")
            .split("://")
            .map(str::to_string);
        Url {
            scheme: parts.next().expect("missing scheme").to_string(),
            host: parts.next().expect("missing host").to_string(),
        }
    }

    fn output(&self, buffer: &mut StringInfo) {
        let res = format!("{}://{}", self.scheme, self.host);
        buffer.write(res.as_bytes()).unwrap();
    }
}

#[pg_extern]
fn is_secure(url: Url) -> bool {
    url.scheme.ends_with("s")
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