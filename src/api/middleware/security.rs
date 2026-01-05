//! Security middleware for API protection
//!
//! This module provides security middleware to protect against common
//! web vulnerabilities as defined by OWASP guidelines.

use axum::{
    body::Body,
    http::{header::HeaderName, HeaderValue, Request, Response},
    middleware::Next,
};

/// Security headers middleware
///
/// # OWASP API8:2023 - Security Misconfiguration
///
/// Adds security headers to all responses to protect against common attacks:
/// - X-Content-Type-Options: nosniff - Prevents MIME type sniffing
/// - X-Frame-Options: DENY - Prevents clickjacking attacks
/// - X-XSS-Protection: 1; mode=block - Enables XSS filter in older browsers
/// - Strict-Transport-Security: Enforces HTTPS connections
/// - Content-Security-Policy: Prevents XSS and data injection attacks
/// - Referrer-Policy: Controls referrer information leakage
/// - Permissions-Policy: Restricts browser features
///
/// # Example
/// ```rust
/// use axum::{Router, middleware};
/// use metamcp::api::middleware::security_headers;
///
/// let app = Router::new()
///     .layer(middleware::from_fn(security_headers));
/// ```
pub async fn security_headers(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // OWASP API8:2023 - X-Content-Type-Options
    // Prevents browsers from MIME-sniffing a response away from the declared content-type
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // OWASP API8:2023 - X-Frame-Options
    // Prevents clickjacking by disallowing the page to be embedded in frames
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // OWASP API8:2023 - X-XSS-Protection
    // Enables XSS filter in older browsers (modern browsers use CSP instead)
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // OWASP API8:2023 - Strict-Transport-Security (HSTS)
    // Forces HTTPS for 1 year, including subdomains
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // OWASP API8:2023 - Content-Security-Policy
    // Restricts resource loading to prevent XSS attacks
    // For APIs, we use a strict policy that only allows self-origin
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'; form-action 'self'"),
    );

    // OWASP API8:2023 - Referrer-Policy
    // Controls how much referrer information is included with requests
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // OWASP API8:2023 - Permissions-Policy (formerly Feature-Policy)
    // Restricts which browser features can be used
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    // OWASP API8:2023 - Cache-Control for sensitive API responses
    // Prevents caching of sensitive data
    headers.insert(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
    );

    // OWASP API8:2023 - Pragma (for HTTP/1.0 compatibility)
    headers.insert(
        HeaderName::from_static("pragma"),
        HeaderValue::from_static("no-cache"),
    );

    response
}

/// Rate limit headers middleware
///
/// # OWASP API4:2023 - Unrestricted Resource Consumption
///
/// Adds rate limiting headers to responses to inform clients about limits.
/// This is a placeholder that adds standard headers - actual rate limiting
/// logic should be implemented using tower-governor or similar.
///
/// Headers added:
/// - X-RateLimit-Limit: Maximum requests per window
/// - X-RateLimit-Remaining: Remaining requests in current window
/// - X-RateLimit-Reset: Unix timestamp when the window resets
pub async fn rate_limit_headers(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // OWASP API4:2023 - Rate limiting headers
    // These are informational headers indicating rate limit policy
    // Actual rate limiting is enforced by the rate limiter middleware
    headers.insert(
        HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from_static("100"),
    );

    // In a real implementation, these would be calculated dynamically
    headers.insert(
        HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from_static("99"),
    );

    // Reset time (1 minute from now - placeholder)
    headers.insert(
        HeaderName::from_static("x-ratelimit-reset"),
        HeaderValue::from_static("60"),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn handler() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn test_security_headers_added() {
        let app = Router::new()
            .route("/", get(handler))
            .layer(axum::middleware::from_fn(security_headers));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
        assert_eq!(
            response.headers().get("x-frame-options").unwrap(),
            "DENY"
        );
        assert_eq!(
            response.headers().get("x-xss-protection").unwrap(),
            "1; mode=block"
        );
    }
}
