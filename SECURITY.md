# Security Policy

We take security vulnerabilities seriously. Thank you for helping us keep WhatsUp secure for everyone.

## Supported Versions

Only the latest release versions are supported with active security patches.

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0.0 | :x:                |

---

## Reporting a Vulnerability

**Please do NOT open a public issue for security-related bugs.**

To report security issues or potential vulnerabilities, email **security@whatsup.dev** with detailed replication steps, system configurations, and proof-of-concept logs.

We will acknowledge receipt within 48 hours and provide monthly status updates during patching steps. Once fixed, we will publish a security advisory.

## Encrypted Access Tokens
All Meta API Cloud tokens and client credentials stored in PostgreSQL database columns are encrypted at the service layer using standard **AES-256-GCM** encryption. Secrets should never be committed into Git logs.
