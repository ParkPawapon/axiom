use std::net::IpAddr;

use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const MAX_DOMAIN_LENGTH: usize = 253;
const ALLOWED_LOCAL_SUFFIXES: &[&str] = &[".test", ".localhost"];

pub fn validate_local_domain(domain: &str) -> AppResult<String> {
    let domain = domain.trim().trim_end_matches('.').to_ascii_lowercase();

    if domain.is_empty() {
        return Err(AppError::Validation(
            "local domain must not be empty".to_string(),
        ));
    }

    if domain.len() > MAX_DOMAIN_LENGTH {
        return Err(AppError::Validation(format!(
            "local domain must be {MAX_DOMAIN_LENGTH} characters or fewer"
        )));
    }

    if domain
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(AppError::Validation(
            "local domain must not contain whitespace or control characters".to_string(),
        ));
    }

    if domain.contains('/') || domain.contains('\\') || domain.contains(':') {
        return Err(AppError::Validation(
            "local domain must not contain URL or path separators".to_string(),
        ));
    }

    if !ALLOWED_LOCAL_SUFFIXES
        .iter()
        .any(|suffix| domain.ends_with(suffix))
    {
        return Err(AppError::Validation(format!(
            "local domain must end with {}",
            ALLOWED_LOCAL_SUFFIXES.join(" or ")
        )));
    }

    for label in domain.split('.') {
        if label.is_empty() {
            return Err(AppError::Validation(
                "local domain labels must not be empty".to_string(),
            ));
        }

        if label.len() > 63 {
            return Err(AppError::Validation(
                "local domain labels must be 63 characters or fewer".to_string(),
            ));
        }

        if label.starts_with('-') || label.ends_with('-') {
            return Err(AppError::Validation(
                "local domain labels must not start or end with a hyphen".to_string(),
            ));
        }

        if !label
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
        {
            return Err(AppError::Validation(
                "local domain labels must contain only letters, numbers, and hyphens".to_string(),
            ));
        }
    }

    Ok(domain)
}

pub fn validate_loopback_address(address: &str) -> AppResult<String> {
    let address = address.trim();
    let parsed: IpAddr = address
        .parse()
        .map_err(|_| AppError::Validation("host address must be a valid IP address".to_string()))?;

    if !parsed.is_loopback() {
        return Err(AppError::Validation(
            "host address must be a loopback address".to_string(),
        ));
    }

    Ok(address.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_test_domains() {
        assert_eq!(
            validate_local_domain("Example.test").expect("domain should validate"),
            "example.test"
        );
    }

    #[test]
    fn rejects_public_domains() {
        let result = validate_local_domain("example.com");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn rejects_non_loopback_addresses() {
        let result = validate_loopback_address("192.168.1.10");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
