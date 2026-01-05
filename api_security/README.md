# MetaMCP API Security Testing Tool

A Rust binary for testing MetaMCP APIs against **OWASP Top 10 API Security Risks 2023**.

## Overview

This tool performs automated security testing against MetaMCP endpoints to identify potential vulnerabilities based on the OWASP API Security Top 10.

## OWASP Top 10 API Security Risks 2023

| Risk | Description | Test Coverage |
|------|-------------|---------------|
| API1 | Broken Object Level Authorization (BOLA) | Yes |
| API2 | Broken Authentication | Yes |
| API3 | Broken Object Property Level Authorization | Yes |
| API4 | Unrestricted Resource Consumption | Yes |
| API5 | Broken Function Level Authorization | Yes |
| API6 | Unrestricted Access to Sensitive Business Flows | Partial |
| API7 | Server Side Request Forgery (SSRF) | Yes |
| API8 | Security Misconfiguration | Yes |
| API9 | Improper Inventory Management | Manual |
| API10 | Unsafe Consumption of APIs | Manual |

## Installation

```bash
cd api_security
cargo build --release
```

## Usage

### Run All Tests

```bash
cargo run -- --base-url http://localhost:3000 --api-key your_api_key
```

### Run Specific Test Category

```bash
# Authentication tests
cargo run -- -b http://localhost:3000 -k your_api_key -t auth

# BOLA tests
cargo run -- -b http://localhost:3000 -k your_api_key -t bola

# SSRF tests
cargo run -- -b http://localhost:3000 -k your_api_key -t ssrf

# Security configuration tests
cargo run -- -b http://localhost:3000 -k your_api_key -t config
```

### Available Test Categories

| Flag | Category | Description |
|------|----------|-------------|
| `auth` | Authentication | API key validation, JWT handling |
| `bola` | BOLA | Object-level authorization |
| `bopla` | BOPLA | Property-level authorization |
| `resource` | Resource | Rate limiting, payload size |
| `fla` | FLA | Function-level authorization |
| `ssrf` | SSRF | Server-side request forgery |
| `config` | Config | Security headers, CORS |
| `all` | All | Run all test categories |

### Command Line Options

```
Options:
  -b, --base-url <BASE_URL>  Base URL of the MetaMCP API [default: http://localhost:3000]
  -k, --api-key <API_KEY>    API key for authentication
  -t, --test <TEST>          Specific test to run [default: all]
  -v, --verbose              Verbose output
  -h, --help                 Print help
  -V, --version              Print version
```

## Example Output

```
MetaMCP API Security Tester
Based on OWASP Top 10 API Security Risks 2023
Target: http://localhost:3000

=== API2:2023 - Broken Authentication ===
[PASS] [CRITICAL] AUTH: Authentication with invalid API key
[PASS] [HIGH] AUTH: Authentication with empty API key
[PASS] [CRITICAL] AUTH: Authentication with malformed JWT
[PASS] [CRITICAL] AUTH: Authentication with expired JWT
[PASS] [CRITICAL] AUTH: SQL injection in API key field

=== API1:2023 - Broken Object Level Authorization (BOLA) ===
[PASS] [CRITICAL] BOLA: Unauthenticated access to protected resource
[PASS] [HIGH] BOLA: Access non-existent resource by ID
[PASS] [CRITICAL] BOLA: Access potentially other user's resource
[PASS] [MEDIUM] BOLA: Access resource with negative ID

=== Security Test Summary ===
Total tests: 25
Passed: 23
Failed: 2

Critical failures: 0
High severity failures: 0

All security tests passed!
```

## Test Severity Levels

- **CRITICAL**: Immediate security risk, must be fixed
- **HIGH**: Significant security risk, should be fixed soon
- **MEDIUM**: Moderate risk, plan to fix
- **LOW**: Minor issue, fix when convenient
- **INFO**: Informational, no action required

## Documentation

See [OWASP_API_SECURITY.md](./OWASP_API_SECURITY.md) for detailed documentation on:
- Each OWASP API Security Risk
- How MetaMCP addresses each risk
- Code references and examples
- Recommendations for improvement

## References

- [OWASP API Security Top 10 2023](https://owasp.org/API-Security/editions/2023/en/0x11-t10/)
- [OWASP API Security Project](https://owasp.org/www-project-api-security/)
