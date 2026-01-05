//! Security utilities for API protection
//!
//! This module provides security validation functions to protect against
//! common API vulnerabilities as defined by OWASP API Security Top 10.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use thiserror::Error;

/// Security validation errors
///
/// # OWASP API7:2023 - Server Side Request Forgery (SSRF)
/// These errors are returned when URL validation detects potential SSRF attacks.
#[derive(Debug, Error)]
pub enum UrlValidationError {
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    #[error("URL scheme '{0}' is not allowed. Only http and https are permitted")]
    InvalidScheme(String),

    #[error("Localhost URLs are not allowed for security reasons")]
    LocalhostBlocked,

    #[error("Private IP addresses are not allowed for security reasons")]
    PrivateIpBlocked,

    #[error("Cloud metadata endpoints are blocked for security reasons")]
    MetadataEndpointBlocked,

    #[error("Link-local addresses are not allowed for security reasons")]
    LinkLocalBlocked,

    #[error("URL host could not be resolved")]
    UnresolvableHost,
}

/// Validates a URL for SSRF vulnerabilities
///
/// # OWASP API7:2023 - Server Side Request Forgery (SSRF)
///
/// This function validates URLs to prevent SSRF attacks by blocking:
/// - Localhost/loopback addresses (127.0.0.1, ::1, localhost)
/// - Private IP ranges (10.x.x.x, 172.16-31.x.x, 192.168.x.x)
/// - Link-local addresses (169.254.x.x, fe80::)
/// - Cloud metadata endpoints (169.254.169.254, metadata.google.internal)
/// - Non-HTTP(S) schemes
///
/// # Arguments
/// * `url_str` - The URL string to validate
///
/// # Returns
/// * `Ok(())` if the URL is safe
/// * `Err(UrlValidationError)` if the URL is potentially dangerous
///
/// # Example
/// ```
/// use metamcp::utils::security::validate_url_for_ssrf;
///
/// // Safe URL
/// assert!(validate_url_for_ssrf("https://api.example.com").is_ok());
///
/// // Blocked URLs
/// assert!(validate_url_for_ssrf("http://localhost:8080").is_err());
/// assert!(validate_url_for_ssrf("http://192.168.1.1").is_err());
/// assert!(validate_url_for_ssrf("http://169.254.169.254/latest/meta-data/").is_err());
/// ```
pub fn validate_url_for_ssrf(url_str: &str) -> Result<(), UrlValidationError> {
    // Parse URL
    let url = url::Url::parse(url_str)
        .map_err(|e| UrlValidationError::InvalidUrl(e.to_string()))?;

    // OWASP API7:2023 - Validate scheme (only allow http/https)
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(UrlValidationError::InvalidScheme(scheme.to_string()));
    }

    // Get host
    let host = url.host_str()
        .ok_or_else(|| UrlValidationError::InvalidUrl("No host in URL".to_string()))?;

    // OWASP API7:2023 - Block localhost variants
    if is_localhost(host) {
        return Err(UrlValidationError::LocalhostBlocked);
    }

    // OWASP API7:2023 - Block cloud metadata endpoints
    if is_cloud_metadata_endpoint(host) {
        return Err(UrlValidationError::MetadataEndpointBlocked);
    }

    // OWASP API7:2023 - Check if host is an IP address
    if let Ok(ip) = host.parse::<IpAddr>() {
        validate_ip_address(&ip)?;
    } else {
        // For hostnames, try to resolve and validate the IP
        // This prevents DNS rebinding attacks
        validate_hostname(host, url.port())?;
    }

    Ok(())
}

/// Check if a host string represents localhost
///
/// # OWASP API7:2023 - SSRF Prevention
fn is_localhost(host: &str) -> bool {
    let host_lower = host.to_lowercase();

    // Check common localhost variants
    host_lower == "localhost"
        || host_lower == "localhost.localdomain"
        || host_lower == "127.0.0.1"
        || host_lower == "::1"
        || host_lower == "[::1]"
        || host_lower == "0.0.0.0"
        || host_lower.starts_with("127.")
        || host_lower.ends_with(".localhost")
        || host_lower.ends_with(".local")
}

/// Check if a host is a cloud metadata endpoint
///
/// # OWASP API7:2023 - SSRF Prevention
/// Cloud metadata services are a common SSRF target for credential theft.
fn is_cloud_metadata_endpoint(host: &str) -> bool {
    let host_lower = host.to_lowercase();

    // AWS EC2 metadata
    host_lower == "169.254.169.254"
        // GCP metadata
        || host_lower == "metadata.google.internal"
        || host_lower == "metadata.goog"
        // Azure metadata
        || host_lower == "169.254.169.254"
        // Kubernetes
        || host_lower == "kubernetes.default"
        || host_lower == "kubernetes.default.svc"
        // Docker
        || host_lower == "host.docker.internal"
        // Generic internal endpoints
        || host_lower.ends_with(".internal")
        || host_lower.ends_with(".local")
}

/// Validate an IP address for SSRF vulnerabilities
///
/// # OWASP API7:2023 - SSRF Prevention
fn validate_ip_address(ip: &IpAddr) -> Result<(), UrlValidationError> {
    match ip {
        IpAddr::V4(ipv4) => validate_ipv4(ipv4),
        IpAddr::V6(ipv6) => validate_ipv6(ipv6),
    }
}

/// Validate an IPv4 address
///
/// # OWASP API7:2023 - SSRF Prevention
/// Blocks private, loopback, link-local, and other non-public IP ranges.
fn validate_ipv4(ip: &Ipv4Addr) -> Result<(), UrlValidationError> {
    // Loopback (127.0.0.0/8)
    if ip.is_loopback() {
        return Err(UrlValidationError::LocalhostBlocked);
    }

    // Private ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
    if ip.is_private() {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    // Link-local (169.254.0.0/16) - includes AWS metadata IP
    if ip.is_link_local() {
        return Err(UrlValidationError::LinkLocalBlocked);
    }

    // Unspecified (0.0.0.0)
    if ip.is_unspecified() {
        return Err(UrlValidationError::LocalhostBlocked);
    }

    // Broadcast (255.255.255.255)
    if ip.is_broadcast() {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    // Documentation ranges (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24)
    if ip.is_documentation() {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    // Additional private/reserved ranges not covered by std::net
    let octets = ip.octets();

    // 100.64.0.0/10 (Carrier-grade NAT)
    if octets[0] == 100 && (octets[1] >= 64 && octets[1] <= 127) {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    // 192.0.0.0/24 (IETF Protocol Assignments)
    if octets[0] == 192 && octets[1] == 0 && octets[2] == 0 {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    // 198.18.0.0/15 (Benchmarking)
    if octets[0] == 198 && (octets[1] == 18 || octets[1] == 19) {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    // 224.0.0.0/4 (Multicast)
    if octets[0] >= 224 && octets[0] <= 239 {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    // 240.0.0.0/4 (Reserved for future use)
    if octets[0] >= 240 {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    Ok(())
}

/// Validate an IPv6 address
///
/// # OWASP API7:2023 - SSRF Prevention
fn validate_ipv6(ip: &Ipv6Addr) -> Result<(), UrlValidationError> {
    // Loopback (::1)
    if ip.is_loopback() {
        return Err(UrlValidationError::LocalhostBlocked);
    }

    // Unspecified (::)
    if ip.is_unspecified() {
        return Err(UrlValidationError::LocalhostBlocked);
    }

    // Check for IPv4-mapped IPv6 addresses (::ffff:x.x.x.x)
    if let Some(ipv4) = ip.to_ipv4_mapped() {
        return validate_ipv4(&ipv4);
    }

    // Unique local addresses (fc00::/7) - IPv6 equivalent of private
    let segments = ip.segments();
    if (segments[0] & 0xfe00) == 0xfc00 {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    // Link-local (fe80::/10)
    if (segments[0] & 0xffc0) == 0xfe80 {
        return Err(UrlValidationError::LinkLocalBlocked);
    }

    // Multicast (ff00::/8)
    if (segments[0] & 0xff00) == 0xff00 {
        return Err(UrlValidationError::PrivateIpBlocked);
    }

    Ok(())
}

/// Validate a hostname by resolving it and checking the IP
///
/// # OWASP API7:2023 - SSRF Prevention
/// This prevents DNS rebinding attacks by validating resolved IPs.
fn validate_hostname(host: &str, port: Option<u16>) -> Result<(), UrlValidationError> {
    // Create socket address for resolution
    let addr_str = format!("{}:{}", host, port.unwrap_or(80));

    // Try to resolve the hostname
    match addr_str.to_socket_addrs() {
        Ok(addrs) => {
            // Check all resolved addresses
            for addr in addrs {
                validate_ip_address(&addr.ip())?;
            }
            Ok(())
        }
        Err(_) => {
            // If we can't resolve, it might be an internal hostname
            // For security, we allow it but log a warning
            // In a stricter mode, you could return UnresolvableHost error
            tracing::warn!(
                "OWASP API7:2023 - Could not resolve hostname '{}', allowing for now",
                host
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(validate_url_for_ssrf("https://api.example.com").is_ok());
        assert!(validate_url_for_ssrf("http://example.com:8080/path").is_ok());
        assert!(validate_url_for_ssrf("https://sub.domain.example.com").is_ok());
    }

    #[test]
    fn test_localhost_blocked() {
        assert!(matches!(
            validate_url_for_ssrf("http://localhost:8080"),
            Err(UrlValidationError::LocalhostBlocked)
        ));
        assert!(matches!(
            validate_url_for_ssrf("http://127.0.0.1:3000"),
            Err(UrlValidationError::LocalhostBlocked)
        ));
        assert!(matches!(
            validate_url_for_ssrf("http://[::1]:8080"),
            Err(UrlValidationError::LocalhostBlocked)
        ));
    }

    #[test]
    fn test_private_ips_blocked() {
        assert!(matches!(
            validate_url_for_ssrf("http://10.0.0.1"),
            Err(UrlValidationError::PrivateIpBlocked)
        ));
        assert!(matches!(
            validate_url_for_ssrf("http://192.168.1.1"),
            Err(UrlValidationError::PrivateIpBlocked)
        ));
        assert!(matches!(
            validate_url_for_ssrf("http://172.16.0.1"),
            Err(UrlValidationError::PrivateIpBlocked)
        ));
    }

    #[test]
    fn test_cloud_metadata_blocked() {
        assert!(matches!(
            validate_url_for_ssrf("http://169.254.169.254/latest/meta-data/"),
            Err(UrlValidationError::LinkLocalBlocked)
        ));
        assert!(matches!(
            validate_url_for_ssrf("http://metadata.google.internal"),
            Err(UrlValidationError::MetadataEndpointBlocked)
        ));
    }

    #[test]
    fn test_invalid_scheme_blocked() {
        assert!(matches!(
            validate_url_for_ssrf("ftp://example.com"),
            Err(UrlValidationError::InvalidScheme(_))
        ));
        assert!(matches!(
            validate_url_for_ssrf("file:///etc/passwd"),
            Err(UrlValidationError::InvalidScheme(_))
        ));
    }
}
