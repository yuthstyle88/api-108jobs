<div align="center">

[![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/app_108jobsNet/app_108jobs.svg)](https://github.com/app_108jobsNet/app_108jobs/releases)
[![Build Status](https://woodpecker.join-app_108jobs.org/api/badges/app_108jobsNet/app_108jobs/status.svg)](https://woodpecker.join-app_108jobs.org/app_108jobsNet/app_108jobs)
[![GitHub issues](https://img.shields.io/github/issues-raw/app_108jobsNet/app_108jobs.svg)](https://github.com/app_108jobsNet/app_108jobs/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/app_108jobs.svg)](https://cloud.docker.com/repository/docker/dessalines/app_108jobs/)
[![Translation status](http://weblate.join-app_108jobs.org/widgets/app_108jobs/-/app_108jobs/svg-badge.svg)](http://weblate.join-app_108jobs.org/engage/app_108jobs/)
[![License](https://img.shields.io/github/license/app_108jobsNet/app_108jobs.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/app_108jobsNet/app_108jobs?style=social)](https://github.com/app_108jobsNet/app_108jobs/stargazers)
<a href="https://endsoftwarepatents.org/innovating-without-patents"><img style="height: 20px;" src="https://static.fsf.org/nosvn/esp/logos/patent-free.svg"></a>

</div>

<p align="center">
  <span>English</span> |
  <a href="readmes/README.es.md">Español</a> |
  <a href="readmes/README.ru.md">Русский</a> |
  <a href="readmes/README.zh.hans.md">汉语</a> |
  <a href="readmes/README.zh.hant.md">漢語</a> |
  <a href="readmes/README.ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://join-app_108jobs.org/" rel="noopener">
 <img width=200px height=200px src="https://raw.githubusercontent.com/app_108jobsNet/app_108jobs-ui/main/src/assets/icons/favicon.svg"></a>

 <h3 align="center"><a href="https://join-app_108jobs.org">app_108jobs</a></h3>
  <p align="center">
    A link aggregator and forum for the fediverse.
    <br />
    <br />
    <a href="https://join-app_108jobs.org">Join app_108jobs</a>
    ·
    <a href="https://join-app_108jobs.org/docs/index.html">Documentation</a>
    ·
    <a href="https://matrix.to/#/#app_108jobs-space:matrix.org">Matrix Chat</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/issues">Report Bug</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/issues">Request Feature</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/blob/main/RELEASES.md">Releases</a>
    ·
    <a href="https://join-app_108jobs.org/docs/code_of_conduct.html">Code of Conduct</a>
  </p>
</p>

## About The Project

| Desktop                                                                                                         | Mobile                                                                                                      |
| --------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| ![desktop](https://raw.githubusercontent.com/app_108jobsNet/joinapp_108jobs-site/main/src/assets/images/main_screen_2.webp) | ![mobile](https://raw.githubusercontent.com/app_108jobsNet/joinapp_108jobs-site/main/src/assets/images/mobile_pic.webp) |

[app_108jobs](https://github.com/app_108jobsNet/app_108jobs) is similar to sites like [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), or [Hacker News](https://news.ycombinator.com/): you subscribe to forums you're interested in, post links and discussions, then vote, and comment on them. Behind the scenes, it is very different; anyone can easily run a server, and all these servers are federated (think email), and connected to the same universe, called the [Fediverse](https://en.wikipedia.org/wiki/Fediverse).

For a link aggregator, this means a user registered on one server can subscribe to forums on any other server, and can have discussions with users registered elsewhere.

It is an easily self-hostable, decentralized alternative to Reddit and other link aggregators, outside of their corporate control and meddling.

Each app_108jobs server can set its own moderation policy; appointing site-wide admins, and community moderators to keep out the trolls, and foster a healthy, non-toxic environment where all can feel comfortable contributing.

### Why's it called app_108jobs?

- Lead singer from [Motörhead](https://invidio.us/watch?v=3mbvWn1EY6g).
- The old school [video game](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>).
- The [Koopa from Super Mario](https://www.mariowiki.com/app_108jobs_Koopa).
- The [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

### Built With

- [Rust](https://www.rust-lang.org)
- [Actix](https://actix.rs/)
- [Diesel](http://diesel.rs/)
- [Inferno](https://infernojs.org)
- [Typescript](https://www.typescriptlang.org/)

## Features

- Open source, [AGPL License](/LICENSE).
- Self hostable, easy to deploy.
  - Comes with [Docker](https://join-app_108jobs.org/docs/administration/install_docker.html) and [Ansible](https://join-app_108jobs.org/docs/administration/install_ansible.html).
- Clean, mobile-friendly interface.
  - Only a minimum of a username and password is required to sign up!
  - User avatar support.
  - Live-updating Comment threads.
  - Full vote scores `(+/-)` like old Reddit.
  - Themes, including light, dark, and solarized.
  - Emojis with autocomplete support. Start typing `:`
  - User tagging using `@`, Community tagging using `!`.
  - Integrated image uploading in both posts and comments.
  - A post can consist of a title and any combination of self text, a URL, or nothing else.
  - Notifications, on comment replies and when you're tagged.
    - Notifications can be sent via email.
    - Private messaging support.
  - i18n / internationalization support.
  - RSS / Atom feeds for `All`, `Subscribed`, `Inbox`, `User`, and `Community`.
- Cross-posting support.
  - A _similar post search_ when creating new posts. Great for question / answer communities.
- Moderation abilities.
  - Public Moderation Logs.
  - Can sticky posts to the top of communities.
  - Both site admins, and community moderators, who can appoint other moderators.
  - Can lock, remove, and restore posts and comments.
  - Can ban and unban users from communities and the site.
  - Can transfer site and communities to others.
- Can fully erase your data, replacing all posts and comments.
- NSFW post / community support.
- High performance.
  - Server is written in rust.
  - Supports arm64 / Raspberry Pi.

## Installation

- [app_108jobs Administration Docs](https://join-app_108jobs.org/docs/administration/administration.html)

## app_108jobs Projects

- [awesome-app_108jobs - A community driven list of apps and tools for app_108jobs](https://github.com/dbeley/awesome-app_108jobs)

## Support / Donate

app_108jobs is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.

app_108jobs is made possible by a generous grant from the [NLnet foundation](https://nlnet.nl/).

- [Support on Liberapay](https://liberapay.com/app_108jobs).
- [Support on Ko-fi](https://ko-fi.com/app_108jobsnet).
- [Support on OpenCollective](https://opencollective.com/app_108jobs).
- [Support on Patreon](https://www.patreon.com/dessalines).

### Crypto

- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## Contributing

Read the following documentation to setup the development environment and start coding:

- [Contributing instructions](https://join-app_108jobs.org/docs/contributors/01-overview.html)
- [Docker Development](https://join-app_108jobs.org/docs/contributors/03-docker-development.html)
- [Local Development](https://join-app_108jobs.org/docs/contributors/02-local-development.html)

When working on an issue or pull request, you can comment with any questions you may have so that maintainers can answer them. You can also join the [Matrix Development Chat](https://matrix.to/#/#app_108jobsdev:matrix.org) for general assistance.

### Translations

- If you want to help with translating, take a look at [Weblate](https://weblate.join-app_108jobs.org/projects/app_108jobs/). You can also help by [translating the documentation](https://github.com/app_108jobsNet/app_108jobs-docs#adding-a-new-language).

## Community

- [Matrix Space](https://matrix.to/#/#app_108jobs-space:matrix.org)
- [app_108jobs Forum](https://app_108jobs.ml/c/app_108jobs)
- [app_108jobs Support Forum](https://app_108jobs.ml/c/app_108jobs_support)

## Code Mirrors

- [GitHub](https://github.com/app_108jobsNet/app_108jobs)
- [Gitea](https://git.join-app_108jobs.org/app_108jobsNet/app_108jobs)
- [Codeberg](https://codeberg.org/app_108jobsNet/app_108jobs)

## Credits

Logo made by Andy Cuccaro (@andycuccaro) under the CC-BY-SA 4.0 license.
