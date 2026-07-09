---
description: Run comprehensive security audit on Rust dependencies and code
agent: build
---

Run a complete security audit on the project:

1. Execute `cargo audit` to check for known vulnerabilities in dependencies
2. Analyze the dependency tree with `cargo tree` for any suspicious or outdated crates
3. Review code for common security issues:
   - Unsafe code blocks without safety comments
   - Unvalidated user input
   - Potential buffer overflows or integer overflows
   - Use of deprecated or insecure APIs
4. Check for any hardcoded secrets or credentials
5. Review Cargo.toml for dependency versions and recommend updates

Provide a detailed report with severity levels (Critical, High, Medium, Low) and actionable remediation steps.

Respond in Portuguese (Brazil) to the user.
