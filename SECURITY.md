# Security Policy

## Supported versions

studfinder is pre-1.0. Only the latest release line receives fixes.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | yes       |
| < 0.1   | no        |

## Reporting a vulnerability

Please report privately rather than opening a public issue.

Use GitHub's private vulnerability reporting: go to the
[Security tab](https://github.com/pblittle/studfinder/security) and choose
**Report a vulnerability**. That opens a channel visible only to the maintainer.

If that option is not available to you, contact the maintainer through their
GitHub profile at https://github.com/pblittle and ask for a private channel.
Either way, please keep the details out of a public issue until there is a fix.

Include what you did, what happened, and the version or commit you were on. A
proof of concept helps but is not required.

Expect an acknowledgement within a week. If a report is confirmed, the fix and a
credit (if you want one) go out in the next release, and the advisory is
published through GitHub.

## Scope

studfinder reads image files, writes to a local SQLite database, and imports and
exports JSON and CSV. Issues in how it parses untrusted input, or anything that
lets a crafted file escape those boundaries, are in scope.

Vulnerabilities in upstream crates should be reported to those projects.
Dependency advisories that affect studfinder are tracked by `cargo audit` in CI.
