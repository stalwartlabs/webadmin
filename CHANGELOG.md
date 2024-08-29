# Change Log

All notable changes to this project will be documented in this file. This project adheres to [Semantic Versioning](http://semver.org/).

## [0.1.12] - 2024-08-29

## Added
- Dashboard (Enterprise edition only).
- Alerts (Enterprise edition only).
- Fail2ban on failed RCPT and loitering events.
- Blocked domains list.
- Is allowed to sender setting.

### Changed

### Fixed
- Super users can't be added to groups.

## [0.1.11] - 2024-08-20

## Added
- SQL Read replicas (Enterprise edition only).
- Distributed blob store (Enterprise edition only).
- Message delivery history (Enterprise edition only).
- Live tracing (Enterprise edition only).

### Changed

### Fixed
- Manage account dropdown displayed on the wrong part of the page.

## [0.1.10] - 2024-08-08

## Added
- Metrics configuration.
- HTTP endpoint access controls.

### Changed

### Fixed
- Unfiltered data store select options on SQL directory creation (fixes #17).

## [0.1.9] - 2024-08-01

## Added
- More tracing configuration options.
- Custom event levels.

### Changed

### Fixed

## [0.1.8] - 2024-07-07

## Added
- Restore deleted emails (Enterprise edition only).
- Option to purge accounts.

### Changed

### Fixed

## [0.1.7] - 2024-07-01

## Added
- Two-factor authentication with TOTP.
- Application passwords.
- Option to disable accounts.

### Changed

### Fixed

## [0.1.6] - 2024-06-22

## Added
- SPAM training and testing interfaces.
- Webhooks management.
- MTA Hooks management.
- IMAP special use folder configuration.
- Display version number by hovering over the logo.
- Logout button.

### Changed
- Show a modal when settings are saved successfully instead of an alert.

### Fixed
- Accept `reject` and `discard` as valid SPAM scores.
- Redis cluster can't be configured.
- Case-insensitive settings search.

## [0.1.5] - 2024-05-23

## Added
- HTTP Strict Transport Security option
- Cleanup settings
- Strict DKIM setting
- Allowed IP list
- Display disk usage even when quotas are disabled

### Changed
- Discourage RSA-SHA1 key usage
- Master user settings
- Removed DKIM signature length option
- Use a single ARC seal by default
- Add server hostname to SMTP greetings

### Fixed
- `session.auth.require` variable type

## [0.1.4] - 2024-05-13

## Added
- Cluster management section.
- MTA-STS policy management.
- Queued message contents view.
- Master user setting.
- mySQL TLS setting.

### Changed

### Fixed

## [0.1.3] - 2024-04-30

## Added

### Changed

### Fixed
- Missing space in accounts column of Domain list (#5)

## [0.1.2] - 2024-04-17

## Added
- `DNS-01` and `HTTP-01` ACME challenge configuration.

### Changed
- Use rust stable.

### Fixed
- Properly escape URL path components.

## [0.1.1] - 2024-04-12

## Added

### Changed

### Fixed
- IP address mask validation.
- Wrap log if too long, to prevent overflow (#1)
- Incorrect base64 alphabet is used in integrity hashes (trunk issue)

## [0.1.0] - 2024-04-09

First release of the project.

## Added

### Changed

### Fixed
