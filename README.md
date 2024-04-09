<p align="center">
    <a href="https://stalw.art">
    <img src="./img/logo-red.svg" height="150">
    </a>
</p>

<h3 align="center">
  Web-based Administration Interface for Stalwart Mail Server 🛡️
</h3>

<br>

<p align="center">
  <a href="https://github.com/stalwartlabs/mail-server/actions/workflows/build.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/stalwartlabs/mail-server/build.yml?style=flat-square" alt="continuous integration">
  </a>
  &nbsp;
  <a href="https://www.gnu.org/licenses/agpl-3.0">
    <img src="https://img.shields.io/badge/License-AGPL_v3-blue.svg?label=license&style=flat-square" alt="License: AGPL v3">
  </a>
  &nbsp;
  <a href="https://stalw.art/docs/get-started/">
    <img src="https://img.shields.io/badge/read_the-docs-red?style=flat-square" alt="Documentation">
  </a>
</p>
<p align="center">
  <a href="https://mastodon.social/@stalwartlabs">
    <img src="https://img.shields.io/mastodon/follow/109929667531941122?style=flat-square&logo=mastodon&color=%236364ff" alt="Mastodon">
  </a>
  &nbsp;
  <a href="https://twitter.com/stalwartlabs">
    <img src="https://img.shields.io/twitter/follow/stalwartlabs?style=flat-square&logo=twitter" alt="Twitter">
  </a>
</p>
<p align="center">
  <a href="https://discord.gg/jtgtCNj66U">
    <img src="https://img.shields.io/discord/923615863037390889?label=discord&style=flat-square" alt="Discord">
  </a>
  &nbsp;
  <a href="https://matrix.to/#/#stalwart:matrix.org">
    <img src="https://img.shields.io/matrix/stalwartmail%3Amatrix.org?label=matrix&style=flat-square" alt="Matrix">
  </a>
</p>

## Features

**Stalwart Webadmin** is an web-based management tool for Stalwart Mail Server. It is written in Rust using the [Leptos](https://github.com/leptos-rs/leptos) web framework.

Key features:

- Account, domain, group and mailing list management.
- SMTP queue management for messages and outbound DMARC and TLS reports.
- Report visualization interface for received DMARC, TLS-RPT and Failure (ARF) reports.
- Configuration of every aspect of the mail server.
- Log viewer with search and filtering capabilities.
- Self-service password reset and encryption-at-rest key management.

## Screenshots

<img src="./img/screencast-setup.gif">

## Get Started

Stalwart Webadmin is included with Stalwart Mail Server, to install Stalwart Mail Server on your server by following the instructions for your platform:

- [Linux / MacOS](https://stalw.art/docs/install/linux)
- [Windows](https://stalw.art/docs/install/windows)
- [Docker](https://stalw.art/docs/install/docker)

All documentation is available at [stalw.art/docs/get-started](https://stalw.art/docs/get-started).

## Support

If you are having problems running Stalwart Mail Server, you found a bug or just have a question,
do not hesitate to reach us on [Github Discussions](https://github.com/stalwartlabs/mail-server/discussions),
[Reddit](https://www.reddit.com/r/stalwartlabs), [Discord](https://discord.gg/aVQr3jF8jd) or [Matrix](https://matrix.to/#/#stalwart:matrix.org).
Additionally you may become a sponsor to obtain priority support from Stalwart Labs Ltd.

## License

Licensed under the terms of the [GNU Affero General Public License](https://www.gnu.org/licenses/agpl-3.0.en.html) as published by
the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
See [LICENSE](LICENSE) for more details.

You can be released from the requirements of the AGPLv3 license by purchasing
a commercial license. Please contact licensing@stalw.art for more details.
  
## Copyright

Copyright (C) 2024, Stalwart Labs Ltd.
