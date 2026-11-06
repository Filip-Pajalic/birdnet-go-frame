# Security

## Reporting a vulnerability

Please report vulnerabilities privately through GitHub:
[Report a vulnerability](https://github.com/Filip-Pajalic/birdnet-go-frame/security/advisories/new).
Do not open a public issue. You should get a reply within a week.

## Scope and expectations

birdnet-go-frame is meant for a home network. It serves plain HTTP without authentication and
reads BirdNET-Go over plain HTTP. Do not expose it to the internet; put it behind a reverse
proxy with TLS and authentication if you need remote access.

Reports about crashes or resource exhaustion from malformed BirdNET-Go responses or HTTP
requests are in scope and welcome.

Only the latest release is supported.
