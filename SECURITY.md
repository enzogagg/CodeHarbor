# Security Policy

CodeHarbor is currently in early private development.

## Reporting A Vulnerability

Do not open a public issue for sensitive security reports. Contact the maintainer privately with:

- A short description of the issue
- Steps to reproduce
- Affected files or commands
- Expected impact

## Local Safety Notes

- CodeHarbor will execute Docker commands locally.
- Generated environments can mount host directories.
- Never store secrets directly in Compose files or committed templates.
- Review generated Dockerfiles before running workspaces from untrusted templates.
