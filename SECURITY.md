# Security Policy

## Reporting a Vulnerability

This is a small personal project. If you find a security issue, please do not
open a public issue. Instead, report it privately by opening a GitHub issue
with the label `security` and minimal details, or contact the maintainer
through the repository's discussions/contact channels.

Please include:

- A description of the issue and its impact
- Steps to reproduce
- Affected versions

## Supported Versions

Only the latest release is supported. Older releases are not patched.

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |
| Older   | :x:                |

## Scope

cocoBar is a standalone desktop application. It does not make network requests
except checking the GitHub Releases API for updates (which the user triggers).
All user data is stored locally in `%APPDATA%\cocoBar\`.
