//! Methods for managing email data

const DOMAIN_SEPARATOR: &str = "@";
const SUFIX_SEPARATOR: &str = "+";

/// Given an email that may, or may not, be sufixed, returns the actual email without the sufix.
pub fn actual_email(email: &str) -> String {
    if !email.contains(SUFIX_SEPARATOR) {
        return email.to_string();
    }

    let email_parts: Vec<&str> = email.split(SUFIX_SEPARATOR).collect();
    let domain = email_parts
        .get(1)
        .and_then(|sufix| sufix.split(DOMAIN_SEPARATOR).nth(1))
        .unwrap_or_default();

    email_parts
        .first()
        .cloned()
        .map(|username| vec![username, domain].join(DOMAIN_SEPARATOR))
        .unwrap_or_default()
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn actual_email_should_not_fail() {
        struct Test<'a> {
            email: &'a str,
            actual_email: &'a str,
        }

        vec![
            Test {
                email: "username@domain.com",
                actual_email: "username@domain.com",
            },
            Test {
                email: "username+sufix@domain.com",
                actual_email: "username@domain.com",
            },
            Test {
                email: "username+@domain.com",
                actual_email: "username@domain.com",
            },
        ]
        .iter()
        .for_each(|test| {
            assert_eq!(
                super::actual_email(test.email),
                test.actual_email.to_string()
            )
        });
    }
}
