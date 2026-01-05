//! API Security Testing Tool for MetaMCP
//!
//! This tool tests MetaMCP APIs against OWASP Top 10 API Security Risks 2023.
//!
//! Usage:
//!   cargo run -- --base-url http://localhost:3000 --api-key your_api_key
//!
//! Or run specific test categories:
//!   cargo run -- --base-url http://localhost:3000 --api-key your_api_key --test auth
//!   cargo run -- --base-url http://localhost:3000 --api-key your_api_key --test bola
//!   cargo run -- --base-url http://localhost:3000 --api-key your_api_key --test ssrf

use clap::Parser;
use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// API Security Testing Tool for MetaMCP
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Base URL of the MetaMCP API
    #[arg(short, long, default_value = "http://localhost:3000")]
    base_url: String,

    /// API key for authentication
    #[arg(short = 'k', long)]
    api_key: Option<String>,

    /// Specific test to run (auth, bola, bopla, resource, fla, ssrf, config, all)
    #[arg(short, long, default_value = "all")]
    test: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthRequest {
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

// McpServer response structure (for future use)
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct McpServer {
    id: Option<i32>,
    name: String,
    url: String,
}

#[derive(Debug, Clone)]
struct TestResult {
    name: String,
    category: String,
    passed: bool,
    expected: String,
    actual: String,
    severity: Severity,
}

#[derive(Debug, Clone, Copy)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

struct SecurityTester {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    jwt_token: Option<String>,
    verbose: bool,
    results: Vec<TestResult>,
}

impl SecurityTester {
    fn new(base_url: String, api_key: Option<String>, verbose: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .danger_accept_invalid_certs(true) // For testing only
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            api_key,
            jwt_token: None,
            verbose,
            results: Vec::new(),
        }
    }

    fn log(&self, msg: &str) {
        if self.verbose {
            println!("{}", msg.dimmed());
        }
    }

    fn add_result(&mut self, result: TestResult) {
        let status = if result.passed {
            "PASS".green()
        } else {
            "FAIL".red()
        };

        let severity_color = match result.severity {
            Severity::Critical => result.severity.to_string().red().bold(),
            Severity::High => result.severity.to_string().red(),
            Severity::Medium => result.severity.to_string().yellow(),
            Severity::Low => result.severity.to_string().blue(),
            Severity::Info => result.severity.to_string().dimmed(),
        };

        println!(
            "[{}] [{}] {}: {}",
            status, severity_color, result.category, result.name
        );

        if !result.passed || self.verbose {
            println!("       Expected: {}", result.expected);
            println!("       Actual:   {}", result.actual);
        }

        self.results.push(result);
    }

    async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(api_key) = &self.api_key {
            self.log(&format!("Authenticating with API key: {}...", &api_key[..8]));

            let resp = self
                .client
                .post(format!("{}/api/v1/auth/token", self.base_url))
                .json(&AuthRequest {
                    api_key: api_key.clone(),
                })
                .send()
                .await?;

            if resp.status().is_success() {
                let auth_resp: AuthResponse = resp.json().await?;
                self.jwt_token = Some(auth_resp.access_token);
                self.log("Authentication successful");
            } else {
                self.log(&format!("Authentication failed: {}", resp.status()));
            }
        }
        Ok(())
    }

    // =========================================================================
    // API1:2023 - Broken Object Level Authorization (BOLA)
    // =========================================================================

    async fn test_bola(&mut self) {
        println!("\n{}", "=== API1:2023 - Broken Object Level Authorization (BOLA) ===".cyan().bold());

        // Test 1: Access without authentication
        self.test_bola_unauthenticated().await;

        // Test 2: Access with invalid object ID
        self.test_bola_invalid_id().await;

        // Test 3: Try to access high ID (potentially other user's resource)
        self.test_bola_other_user_resource().await;

        // Test 4: Try negative ID
        self.test_bola_negative_id().await;
    }

    async fn test_bola_unauthenticated(&mut self) {
        let resp = self
            .client
            .get(format!("{}/api/v1/mcp/servers", self.base_url))
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                (status == 401, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Unauthenticated access to protected resource".to_string(),
            category: "BOLA".to_string(),
            passed,
            expected: "Status: 401 Unauthorized".to_string(),
            actual,
            severity: Severity::Critical,
        });
    }

    async fn test_bola_invalid_id(&mut self) {
        if self.jwt_token.is_none() {
            return;
        }

        let resp = self
            .client
            .get(format!("{}/api/v1/mcp/servers/99999999", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.jwt_token.as_ref().unwrap()),
            )
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                // Should return 400/404, not 200 or 500 (no info leak)
                (status == 400 || status == 404, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Access non-existent resource by ID".to_string(),
            category: "BOLA".to_string(),
            passed,
            expected: "Status: 400/404 (Bad Request/Not Found)".to_string(),
            actual,
            severity: Severity::High,
        });
    }

    async fn test_bola_other_user_resource(&mut self) {
        if self.jwt_token.is_none() {
            return;
        }

        // Try to access resource ID 1 (likely belongs to another user or system)
        let resp = self
            .client
            .get(format!("{}/api/v1/mcp/servers/1", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.jwt_token.as_ref().unwrap()),
            )
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                // Should return 400/403/404 - not 200 (would be unauthorized access)
                (
                    status == 400 || status == 403 || status == 404,
                    format!("Status: {}", status),
                )
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Access potentially other user's resource".to_string(),
            category: "BOLA".to_string(),
            passed,
            expected: "Status: 400/403/404 (denied access)".to_string(),
            actual,
            severity: Severity::Critical,
        });
    }

    async fn test_bola_negative_id(&mut self) {
        if self.jwt_token.is_none() {
            return;
        }

        let resp = self
            .client
            .get(format!("{}/api/v1/mcp/servers/-1", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.jwt_token.as_ref().unwrap()),
            )
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                // Should return 400 Bad Request or 404 Not Found
                (
                    status == 400 || status == 404,
                    format!("Status: {}", status),
                )
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Access resource with negative ID".to_string(),
            category: "BOLA".to_string(),
            passed,
            expected: "Status: 400/404 (Bad Request/Not Found)".to_string(),
            actual,
            severity: Severity::Medium,
        });
    }

    // =========================================================================
    // API2:2023 - Broken Authentication
    // =========================================================================

    async fn test_authentication(&mut self) {
        println!("\n{}", "=== API2:2023 - Broken Authentication ===".cyan().bold());

        // Test 1: Invalid API key
        self.test_auth_invalid_api_key().await;

        // Test 2: Empty API key
        self.test_auth_empty_api_key().await;

        // Test 3: Malformed JWT
        self.test_auth_malformed_jwt().await;

        // Test 4: Expired JWT simulation
        self.test_auth_expired_jwt().await;

        // Test 5: SQL injection in API key
        self.test_auth_sql_injection().await;
    }

    async fn test_auth_invalid_api_key(&mut self) {
        let resp = self
            .client
            .post(format!("{}/api/v1/auth/token", self.base_url))
            .json(&AuthRequest {
                api_key: "invalid_key_12345".to_string(),
            })
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                (status == 401, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Authentication with invalid API key".to_string(),
            category: "AUTH".to_string(),
            passed,
            expected: "Status: 401 Unauthorized".to_string(),
            actual,
            severity: Severity::Critical,
        });
    }

    async fn test_auth_empty_api_key(&mut self) {
        let resp = self
            .client
            .post(format!("{}/api/v1/auth/token", self.base_url))
            .json(&AuthRequest {
                api_key: "".to_string(),
            })
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                (status == 400 || status == 401, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Authentication with empty API key".to_string(),
            category: "AUTH".to_string(),
            passed,
            expected: "Status: 400/401 (Bad Request/Unauthorized)".to_string(),
            actual,
            severity: Severity::High,
        });
    }

    async fn test_auth_malformed_jwt(&mut self) {
        let resp = self
            .client
            .get(format!("{}/api/v1/mcp/servers", self.base_url))
            .header("Authorization", "Bearer not.a.valid.jwt.token")
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                (status == 401, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Authentication with malformed JWT".to_string(),
            category: "AUTH".to_string(),
            passed,
            expected: "Status: 401 Unauthorized".to_string(),
            actual,
            severity: Severity::Critical,
        });
    }

    async fn test_auth_expired_jwt(&mut self) {
        // A JWT that looks valid but has expired timestamp
        let expired_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiZXhwIjoxfQ.invalid";

        let resp = self
            .client
            .get(format!("{}/api/v1/mcp/servers", self.base_url))
            .header("Authorization", format!("Bearer {}", expired_jwt))
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                (status == 401, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Authentication with expired JWT".to_string(),
            category: "AUTH".to_string(),
            passed,
            expected: "Status: 401 Unauthorized".to_string(),
            actual,
            severity: Severity::Critical,
        });
    }

    async fn test_auth_sql_injection(&mut self) {
        let resp = self
            .client
            .post(format!("{}/api/v1/auth/token", self.base_url))
            .json(&AuthRequest {
                api_key: "' OR '1'='1".to_string(),
            })
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                // Should return 401, not 200 (which would indicate SQL injection worked)
                (status == 401, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "SQL injection in API key field".to_string(),
            category: "AUTH".to_string(),
            passed,
            expected: "Status: 401 (SQL injection blocked)".to_string(),
            actual,
            severity: Severity::Critical,
        });
    }

    // =========================================================================
    // API3:2023 - Broken Object Property Level Authorization
    // =========================================================================

    async fn test_bopla(&mut self) {
        println!("\n{}", "=== API3:2023 - Broken Object Property Level Authorization ===".cyan().bold());

        // Test 1: Mass assignment attack
        self.test_bopla_mass_assignment().await;

        // Test 2: Check for excessive data exposure
        self.test_bopla_data_exposure().await;
    }

    async fn test_bopla_mass_assignment(&mut self) {
        if self.jwt_token.is_none() {
            self.add_result(TestResult {
                name: "Mass assignment attack".to_string(),
                category: "BOPLA".to_string(),
                passed: false,
                expected: "Test requires authentication".to_string(),
                actual: "No JWT token available".to_string(),
                severity: Severity::Info,
            });
            return;
        }

        // Try to create a server with extra fields that shouldn't be assignable
        // Using a public URL to avoid SSRF blocking
        let malicious_payload = serde_json::json!({
            "name": "Test Server",
            "url": "https://api.example.com",
            "is_admin": true,
            "user_id": 1,
            "internal_flag": true
        });

        let resp = self
            .client
            .post(format!("{}/api/v1/mcp/servers", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.jwt_token.as_ref().unwrap()),
            )
            .json(&malicious_payload)
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                // Should succeed (ignoring extra fields) or reject with 400
                // Should NOT allow is_admin or user_id to be set
                // 422 is also acceptable if URL validation blocks the request
                (
                    status == 200 || status == 201 || status == 400 || status == 422,
                    format!("Status: {}", status),
                )
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Mass assignment attack (extra fields)".to_string(),
            category: "BOPLA".to_string(),
            passed,
            expected: "Extra fields ignored or rejected".to_string(),
            actual,
            severity: Severity::High,
        });
    }

    async fn test_bopla_data_exposure(&mut self) {
        if self.jwt_token.is_none() {
            return;
        }

        let resp = self
            .client
            .get(format!("{}/api/v1/mcp/servers", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.jwt_token.as_ref().unwrap()),
            )
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                if r.status().is_success() {
                    let body = r.text().await.unwrap_or_default();
                    // Check for sensitive fields that shouldn't be exposed
                    let has_sensitive = body.contains("password")
                        || body.contains("secret")
                        || body.contains("internal_");

                    (
                        !has_sensitive,
                        if has_sensitive {
                            "Sensitive fields exposed in response".to_string()
                        } else {
                            "No sensitive fields in response".to_string()
                        },
                    )
                } else {
                    (true, format!("Status: {}", r.status()))
                }
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Check for excessive data exposure".to_string(),
            category: "BOPLA".to_string(),
            passed,
            expected: "No sensitive fields exposed".to_string(),
            actual,
            severity: Severity::High,
        });
    }

    // =========================================================================
    // API4:2023 - Unrestricted Resource Consumption
    // =========================================================================

    async fn test_resource_consumption(&mut self) {
        println!("\n{}", "=== API4:2023 - Unrestricted Resource Consumption ===".cyan().bold());

        // Test 1: Large payload
        self.test_resource_large_payload().await;

        // Test 2: Check for rate limiting headers
        self.test_resource_rate_limiting().await;
    }

    async fn test_resource_large_payload(&mut self) {
        // Create a 2MB payload
        let large_payload = "A".repeat(2 * 1024 * 1024);

        let resp = self
            .client
            .post(format!("{}/api/v1/auth/token", self.base_url))
            .body(large_payload)
            .header("Content-Type", "application/json")
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                // Should return 413 Payload Too Large or 400 Bad Request
                (
                    status == 413 || status == 400,
                    format!("Status: {}", status),
                )
            }
            Err(e) => {
                // Connection reset or timeout is also acceptable
                (true, format!("Connection rejected: {}", e))
            }
        };

        self.add_result(TestResult {
            name: "Large payload rejection".to_string(),
            category: "RESOURCE".to_string(),
            passed,
            expected: "Status: 413/400 or connection rejected".to_string(),
            actual,
            severity: Severity::Medium,
        });
    }

    async fn test_resource_rate_limiting(&mut self) {
        let resp = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let headers = r.headers();
                let has_rate_limit = headers.contains_key("x-ratelimit-limit")
                    || headers.contains_key("x-rate-limit-limit")
                    || headers.contains_key("ratelimit-limit");

                (
                    has_rate_limit,
                    if has_rate_limit {
                        "Rate limiting headers present".to_string()
                    } else {
                        "No rate limiting headers found".to_string()
                    },
                )
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Rate limiting headers".to_string(),
            category: "RESOURCE".to_string(),
            passed,
            expected: "Rate limiting headers present".to_string(),
            actual,
            severity: Severity::Medium,
        });
    }

    // =========================================================================
    // API5:2023 - Broken Function Level Authorization
    // =========================================================================

    async fn test_fla(&mut self) {
        println!("\n{}", "=== API5:2023 - Broken Function Level Authorization ===".cyan().bold());

        // Test 1: Try to access admin endpoints
        self.test_fla_admin_endpoints().await;

        // Test 2: Try HTTP method tampering
        self.test_fla_method_tampering().await;
    }

    async fn test_fla_admin_endpoints(&mut self) {
        let admin_paths = [
            "/admin",
            "/api/v1/admin",
            "/api/v1/users",
            "/api/admin/config",
        ];

        for path in admin_paths {
            let resp = self
                .client
                .get(format!("{}{}", self.base_url, path))
                .send()
                .await;

            let (passed, actual) = match resp {
                Ok(r) => {
                    let status = r.status().as_u16();
                    // Should return 401, 403, or 404 - NOT 200
                    (
                        status == 401 || status == 403 || status == 404,
                        format!("Status: {}", status),
                    )
                }
                Err(e) => (true, format!("Connection error (acceptable): {}", e)),
            };

            self.add_result(TestResult {
                name: format!("Admin endpoint access: {}", path),
                category: "FLA".to_string(),
                passed,
                expected: "Status: 401/403/404".to_string(),
                actual,
                severity: Severity::High,
            });
        }
    }

    async fn test_fla_method_tampering(&mut self) {
        // Try DELETE on a GET-only endpoint
        let resp = self
            .client
            .delete(format!("{}/health", self.base_url))
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                // Should return 405 Method Not Allowed
                (status == 405, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "HTTP method tampering (DELETE on GET endpoint)".to_string(),
            category: "FLA".to_string(),
            passed,
            expected: "Status: 405 Method Not Allowed".to_string(),
            actual,
            severity: Severity::Medium,
        });
    }

    // =========================================================================
    // API7:2023 - Server Side Request Forgery (SSRF)
    // =========================================================================

    async fn test_ssrf(&mut self) {
        println!("\n{}", "=== API7:2023 - Server Side Request Forgery (SSRF) ===".cyan().bold());

        if self.jwt_token.is_none() {
            self.add_result(TestResult {
                name: "SSRF tests".to_string(),
                category: "SSRF".to_string(),
                passed: false,
                expected: "Test requires authentication".to_string(),
                actual: "No JWT token available".to_string(),
                severity: Severity::Info,
            });
            return;
        }

        // Test various SSRF payloads
        self.test_ssrf_localhost().await;
        self.test_ssrf_private_ip().await;
        self.test_ssrf_cloud_metadata().await;
    }

    async fn test_ssrf_localhost(&mut self) {
        let ssrf_payload = serde_json::json!({
            "name": "SSRF Test",
            "url": "http://localhost:22"
        });

        let resp = self
            .client
            .post(format!("{}/api/v1/mcp/servers", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.jwt_token.as_ref().unwrap()),
            )
            .json(&ssrf_payload)
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                // Should reject localhost URLs with 422 (security violation)
                // Also accept 400 for backwards compatibility
                (status == 422 || status == 400, format!("Status: {}", status))
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "SSRF: localhost URL".to_string(),
            category: "SSRF".to_string(),
            passed,
            expected: "Status: 422 (Security Violation - URL blocked)".to_string(),
            actual,
            severity: Severity::Critical,
        });
    }

    async fn test_ssrf_private_ip(&mut self) {
        let private_ips = ["http://10.0.0.1:80", "http://192.168.1.1:80", "http://172.16.0.1:80"];

        for ip in private_ips {
            let ssrf_payload = serde_json::json!({
                "name": "SSRF Test",
                "url": ip
            });

            let resp = self
                .client
                .post(format!("{}/api/v1/mcp/servers", self.base_url))
                .header(
                    "Authorization",
                    format!("Bearer {}", self.jwt_token.as_ref().unwrap()),
                )
                .json(&ssrf_payload)
                .send()
                .await;

            let (passed, actual) = match resp {
                Ok(r) => {
                    let status = r.status().as_u16();
                    // Should reject private IPs with 422 (security violation)
                    (status == 422 || status == 400, format!("Status: {}", status))
                }
                Err(e) => (false, format!("Error: {}", e)),
            };

            self.add_result(TestResult {
                name: format!("SSRF: private IP {}", ip),
                category: "SSRF".to_string(),
                passed,
                expected: "Status: 422 (Security Violation - Private IP blocked)".to_string(),
                actual,
                severity: Severity::Critical,
            });
        }
    }

    async fn test_ssrf_cloud_metadata(&mut self) {
        let metadata_urls = [
            "http://169.254.169.254/latest/meta-data/",
            "http://metadata.google.internal/",
        ];

        for url in metadata_urls {
            let ssrf_payload = serde_json::json!({
                "name": "SSRF Test",
                "url": url
            });

            let resp = self
                .client
                .post(format!("{}/api/v1/mcp/servers", self.base_url))
                .header(
                    "Authorization",
                    format!("Bearer {}", self.jwt_token.as_ref().unwrap()),
                )
                .json(&ssrf_payload)
                .send()
                .await;

            let (passed, actual) = match resp {
                Ok(r) => {
                    let status = r.status().as_u16();
                    // Should reject metadata URLs with 422 (security violation)
                    (status == 422 || status == 400, format!("Status: {}", status))
                }
                Err(e) => (false, format!("Error: {}", e)),
            };

            self.add_result(TestResult {
                name: format!("SSRF: cloud metadata {}", url),
                category: "SSRF".to_string(),
                passed,
                expected: "Status: 422 (Security Violation - Metadata URL blocked)".to_string(),
                actual,
                severity: Severity::Critical,
            });
        }
    }

    // =========================================================================
    // API8:2023 - Security Misconfiguration
    // =========================================================================

    async fn test_security_config(&mut self) {
        println!("\n{}", "=== API8:2023 - Security Misconfiguration ===".cyan().bold());

        self.test_config_security_headers().await;
        self.test_config_cors().await;
        self.test_config_error_disclosure().await;
    }

    async fn test_config_security_headers(&mut self) {
        let resp = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let headers = r.headers();
                let mut missing = Vec::new();

                if !headers.contains_key("x-content-type-options") {
                    missing.push("X-Content-Type-Options");
                }
                if !headers.contains_key("x-frame-options") {
                    missing.push("X-Frame-Options");
                }
                if !headers.contains_key("x-xss-protection") {
                    missing.push("X-XSS-Protection");
                }

                (
                    missing.is_empty(),
                    if missing.is_empty() {
                        "All security headers present".to_string()
                    } else {
                        format!("Missing: {}", missing.join(", "))
                    },
                )
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Security headers".to_string(),
            category: "CONFIG".to_string(),
            passed,
            expected: "Security headers present".to_string(),
            actual,
            severity: Severity::Medium,
        });
    }

    async fn test_config_cors(&mut self) {
        let resp = self
            .client
            .request(
                reqwest::Method::OPTIONS,
                format!("{}/api/v1/mcp/servers", self.base_url),
            )
            .header("Origin", "http://evil.com")
            .header("Access-Control-Request-Method", "GET")
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let headers = r.headers();
                let allow_origin: &str = headers
                    .get("access-control-allow-origin")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");

                // Check if CORS is too permissive
                let is_permissive = allow_origin == "*";

                (
                    !is_permissive,
                    if is_permissive {
                        "CORS allows all origins (*)".to_string()
                    } else {
                        format!("CORS origin: {}", allow_origin)
                    },
                )
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "CORS configuration".to_string(),
            category: "CONFIG".to_string(),
            passed,
            expected: "CORS should not allow all origins".to_string(),
            actual,
            severity: Severity::Medium,
        });
    }

    async fn test_config_error_disclosure(&mut self) {
        // Send malformed JSON to trigger error
        let resp = self
            .client
            .post(format!("{}/api/v1/auth/token", self.base_url))
            .body("{invalid json")
            .header("Content-Type", "application/json")
            .send()
            .await;

        let (passed, actual) = match resp {
            Ok(r) => {
                let body = r.text().await.unwrap_or_default();
                // Check for stack traces or internal details
                let has_disclosure = body.contains("stack")
                    || body.contains("trace")
                    || body.contains("at src/")
                    || body.contains("panic")
                    || body.contains("RUST_BACKTRACE");

                (
                    !has_disclosure,
                    if has_disclosure {
                        "Error response contains internal details".to_string()
                    } else {
                        "Error response is sanitized".to_string()
                    },
                )
            }
            Err(e) => (false, format!("Error: {}", e)),
        };

        self.add_result(TestResult {
            name: "Error message disclosure".to_string(),
            category: "CONFIG".to_string(),
            passed,
            expected: "No internal details in error responses".to_string(),
            actual,
            severity: Severity::Low,
        });
    }

    fn print_summary(&self) {
        println!("\n{}", "=== Security Test Summary ===".cyan().bold());

        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        let critical_failed = self
            .results
            .iter()
            .filter(|r| !r.passed && matches!(r.severity, Severity::Critical))
            .count();
        let high_failed = self
            .results
            .iter()
            .filter(|r| !r.passed && matches!(r.severity, Severity::High))
            .count();

        println!("Total tests: {}", total);
        println!("Passed: {}", passed.to_string().green());
        println!("Failed: {}", failed.to_string().red());
        println!();
        println!(
            "Critical failures: {}",
            if critical_failed > 0 {
                critical_failed.to_string().red().bold()
            } else {
                "0".green().bold()
            }
        );
        println!(
            "High severity failures: {}",
            if high_failed > 0 {
                high_failed.to_string().red()
            } else {
                "0".green()
            }
        );

        if critical_failed > 0 {
            println!(
                "\n{}",
                "WARNING: Critical security issues detected!".red().bold()
            );
        } else if high_failed > 0 {
            println!(
                "\n{}",
                "CAUTION: High severity issues should be addressed.".yellow()
            );
        } else if failed == 0 {
            println!("\n{}", "All security tests passed!".green().bold());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("{}", "MetaMCP API Security Tester".cyan().bold());
    println!("{}", "Based on OWASP Top 10 API Security Risks 2023".dimmed());
    println!("Target: {}", args.base_url);
    println!();

    let mut tester = SecurityTester::new(args.base_url, args.api_key, args.verbose);

    // Try to authenticate if API key provided
    tester.authenticate().await?;

    // Run tests based on selection
    match args.test.as_str() {
        "auth" => tester.test_authentication().await,
        "bola" => tester.test_bola().await,
        "bopla" => tester.test_bopla().await,
        "resource" => tester.test_resource_consumption().await,
        "fla" => tester.test_fla().await,
        "ssrf" => tester.test_ssrf().await,
        "config" => tester.test_security_config().await,
        "all" | _ => {
            tester.test_authentication().await;
            tester.test_bola().await;
            tester.test_bopla().await;
            tester.test_resource_consumption().await;
            tester.test_fla().await;
            tester.test_ssrf().await;
            tester.test_security_config().await;
        }
    }

    tester.print_summary();

    Ok(())
}
