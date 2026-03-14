---
name: security-scan
description: Scan code for OWASP vulnerabilities and security issues.
  Use for security-sensitive implementations.
---

# Security Scan Skill

## Purpose
Identify and prevent security vulnerabilities.

## OWASP Top 10 Checklist
Reference: [security-scan/checklists/owasp-top-10.md](security-scan/checklists/owasp-top-10.md)

### A01: Broken Access Control
- [ ] Authorization on all endpoints
- [ ] Deny by default
- [ ] Rate limiting implemented
- [ ] CORS properly configured

### A02: Cryptographic Failures
- [ ] Data encrypted in transit (HTTPS)
- [ ] Sensitive data encrypted at rest
- [ ] Strong algorithms used
- [ ] Keys properly managed

### A03: Injection
- [ ] Parameterized queries
- [ ] Input validation
- [ ] Output encoding
- [ ] No eval() with user input

## Authentication Checklist
Reference: [security-scan/checklists/auth-security.md](security-scan/checklists/auth-security.md)
- [ ] Passwords hashed (bcrypt/argon2)
- [ ] Session properly managed
- [ ] Tokens securely stored
- [ ] Logout invalidates session

## Data Validation Checklist
Reference: [security-scan/checklists/data-validation.md](security-scan/checklists/data-validation.md)
- [ ] All input validated
- [ ] Type checking enforced
- [ ] Size limits set
- [ ] Format validation done

## Vulnerability Severity Levels

### Critical
- Remote code execution
- SQL injection
- Authentication bypass
- Sensitive data exposure

### High
- Cross-site scripting (XSS)
- Cross-site request forgery (CSRF)
- Insecure deserialization
- Privilege escalation

### Medium
- Information disclosure
- Missing encryption
- Weak session management
- Insufficient logging

### Low
- Missing security headers
- Verbose error messages
- Outdated dependencies (no known exploits)

## Remediation Process

1. **Critical/High**: Fix immediately, block merge
2. **Medium**: Fix before release
3. **Low**: Track in backlog

## Best Practices

### Do
- Use parameterized queries
- Validate all input
- Encode all output
- Use security headers
- Keep dependencies updated

### Don't
- Hardcode secrets
- Trust user input
- Expose stack traces
- Use weak algorithms
- Skip authentication checks
