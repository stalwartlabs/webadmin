# Change Log

All notable changes to this project will be documented in this file. This project adheres to [Semantic Versioning](http://semver.org/).

## [0.1.27] - 2025-06-04

## Added
- Updated LDAP configuration.

### Changed

### Fixed
- Correct `dav.lock.max-timeout` default value (stalwartlabs/stalwart#1575)
- Hash secrets when updated via forms (credits to @denschub for the report).

## [0.1.26] - 2025-05-26

## Added
- WebDAV, CalDAV amd CardDAV configuration.
- NATS PubSub configuration.

### Changed

### Fixed

## [0.1.25] - 2024-03-40

## Added
- LDAP attribute to indicate password change (stalwartlabs/mail-server#1156)

### Changed

### Fixed
- Make Profile and Security token S3 settings optional (stalwartlabs/mail-server#1166)

## [0.1.24] - 2024-02-01

## Added

### Changed
- Open source third party OIDC support.

### Fixed

## [0.1.23] - 2024-01-29

## Added

### Changed
- Removed free and disposable email providers sections.
- `session.throttle.*` is now `queue.limiter.inbound.*`.
- `queue.throttle.*` is now `queue.limiter.outbound.*`.

### Fixed
- Invalid member counts (stalwartlabs/mail-server#1105).

## [0.1.22] - 2024-01-17

## Added
- Top bar dropdowns.
- Cluster node roles.
- `config_get` expression support.

### Changed
- Renamed to `lookup.default.[hostname|domain]` to `server.hostname` and `report.domain` settings.
- Removed known DMARC list.

### Fixed

## [0.1.21] - 2024-01-06

## Added
- New Spam filter configuration sections and options.

### Changed

### Fixed
- UI improvements handling large lists (stalwartlabs/mail-server#925).
- Remove None from MTA-STS modes (#48).

## [0.1.20] - 2024-12-04

## Added
- Delivery and DMARC troubleshooting.
- Azure blob storage.
- Support for external email addresses on mailing lists.

### Changed
- LDAP/SQL simplified config.

### Fixed

## [0.1.19] - 2024-11-07

## Added
- `bind.auth.search` and `enterprise.api-key` settings.

### Changed

### Fixed

## [0.1.18] - 2024-10-08

## Added
- 'Automatic Ban' section.
- Support for 'External Account Binding' in ACME providers.

### Changed

### Fixed
- Include nonce in OAuth code request.

## [0.1.17] - 2024-10-07

## Added
- AI model management.
- LLM Classifier settings.
- S3 `max-retries` setting.

### Changed

### Fixed
- Allow non-HTTPS `redirect-uri` for OAuth clients.
- Viewport and autocapitalize fixes for mobile devices.
- Fix error reporting in array fields (#25).

## [0.1.16] - 2024-10-01

## Added
- OpenID Connect support.
- API key and OAuth client management.
- Form submission management.

### Changed
- Improved Permission management UI.

### Fixed
- Add permissions tab to group edit screen (stalwartlabs/mail-server#826)
- Include account name in undelete pages (stalwartlabs/mail-server#808)
- Include a Content-Type header when posting (#18).
- Correctly set tenant limits.

## [0.1.15] - 2024-09-20

## Added
- Multi-tenancy support.
- Role & permissions management.
- FTS reindex management action.
- View DNS records in BIND format.

### Changed

### Fixed
- Display memory usage in MBs rather than bytes.

## [0.1.14] - 2024-09-09

## Added

### Changed
- Add `config.local-keys.*` to default local keys.

### Fixed
- Fix: Unable to disable optional duration and rate settings.
- Fix: Externally update selected values.
- Fix: Dashboard averages all chart values.

## [0.1.13] - 2024-09-02

## Added
- Untrusted Sieve script management.

### Changed

### Fixed

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
