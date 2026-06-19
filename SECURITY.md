# Security Policy

## Reporting a vulnerability

Report security issues privately via:

[GitHub Security Advisories](https://github.com/byrdltd/whyLIAN/security/advisories/new)

Please **do not** open public issues for vulnerabilities.

## In scope

- Privilege escalation via udev/systemd misconfiguration shipped by this repo
- Arbitrary code execution through config parsing or IPC socket handling
- Unsafe handling of USB/RF input that could compromise the daemon host
- Supply-chain issues in fork-specific install scripts or CI

## Out of scope

- Bugs in upstream lian-li-linux already reported upstream (report there too)
- Physical USB attacks requiring local device access
- Issues in Lian Li proprietary Windows software

## Supported versions

Security fixes target the latest `main` branch of [byrdltd/whyLIAN](https://github.com/byrdltd/whyLIAN).
