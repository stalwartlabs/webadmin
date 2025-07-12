<p align="center">
    <a href="https://stalw.art">
    <img src="./img/logo-red.svg" height="150">
    </a>
</p>

<h3 align="center">
  Web-based Administration Interface for Stalwart 🛡️
</h3>

<br>

<p align="center">
  <a href="https://github.com/stalwartlabs/webadmin/actions/workflows/build.yml"><img src="https://img.shields.io/github/actions/workflow/status/stalwartlabs/webadmin/build.yml?style=flat-square" alt="continuous integration"></a>
  &nbsp;
  <a href="https://www.gnu.org/licenses/agpl-3.0"><img src="https://img.shields.io/badge/License-AGPL_v3-blue.svg?label=license&style=flat-square" alt="License: AGPL v3"></a>
  &nbsp;
  <a href="https://stalw.art/docs/get-started/"><img src="https://img.shields.io/badge/read_the-docs-red?style=flat-square" alt="Documentation"></a>
</p>
<p align="center">
  <a href="https://mastodon.social/@stalwartlabs"><img src="https://img.shields.io/mastodon/follow/109929667531941122?style=flat-square&logo=mastodon&color=%236364ff&label=Follow%20on%20Mastodon" alt="Mastodon"></a>
  &nbsp;
  <a href="https://twitter.com/stalwartlabs"><img src="https://img.shields.io/twitter/follow/stalwartlabs?style=flat-square&logo=x&label=Follow%20on%20Twitter" alt="Twitter"></a>
</p>
<p align="center">
  <a href="https://discord.gg/jtgtCNj66U"><img src="https://img.shields.io/discord/923615863037390889?label=Join%20Discord&logo=discord&style=flat-square" alt="Discord"></a>
  &nbsp;
  <a href="https://matrix.to/#/#stalwart:matrix.org"><img src="https://img.shields.io/matrix/stalwartmail%3Amatrix.org?label=Join%20Matrix&logo=matrix&style=flat-square" alt="Matrix"></a>
</p>

## Features

**Stalwart Webadmin** is an web-based management tool for Stalwart Mail and Collaboration Server. It is written in Rust using the [Leptos](https://github.com/leptos-rs/leptos) web framework.

Key features:

- Dashboard.
- Account, domain, group, mailing list, tenant and role management.
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
Additionally you may purchase a subscription to obtain priority support from Stalwart Labs Ltd.

## License

This project is dual-licensed under the **GNU Affero General Public License v3.0** (AGPL-3.0; as published by the Free Software Foundation) and the **Stalwart Enterprise License v1 (SELv1)**:

- The [GNU Affero General Public License v3.0](./LICENSES/AGPL-3.0-only.txt) is a free software license that ensures your freedom to use, modify, and distribute the software, with the condition that any modified versions of the software must also be distributed under the same license. 
- The [Stalwart Enterprise License v1 (SELv1)](./LICENSES/LicenseRef-SEL.txt) is a proprietary license designed for commercial use. It offers additional features and greater flexibility for businesses that do not wish to comply with the AGPL-3.0 license requirements. 

Each file in this project contains a license notice at the top, indicating the applicable license(s). The license notice follows the [REUSE guidelines](https://reuse.software/) to ensure clarity and consistency. The full text of each license is available in the [LICENSES](./LICENSES/) directory.
  
## Copyright

Copyright (C) 2024, Stalwart Labs Ltd.
