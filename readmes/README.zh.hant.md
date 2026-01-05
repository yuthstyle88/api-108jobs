<!-- This Chinese variant is generated from ./README.zh.hans.md via OpenCC and then proofread. Regional difference may occur, though. -->
<div align="center">

![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/app_108jobsNet/app_108jobs.svg)
[![Build Status](https://cloud.drone.io/api/badges/app_108jobsNet/app_108jobs/status.svg)](https://cloud.drone.io/app_108jobsNet/app_108jobs/)
[![GitHub issues](https://img.shields.io/github/issues-raw/app_108jobsNet/app_108jobs.svg)](https://github.com/app_108jobsNet/app_108jobs/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/app_108jobs.svg)](https://cloud.docker.com/repository/docker/dessalines/app_108jobs/)
[![Translation status](http://weblate.yerbamate.ml/widgets/app_108jobs/-/app_108jobs/svg-badge.svg)](http://weblate.yerbamate.ml/engage/app_108jobs/)
[![License](https://img.shields.io/github/license/app_108jobsNet/app_108jobs.svg)](LICENSE)
![GitHub stars](https://img.shields.io/github/stars/app_108jobsNet/app_108jobs?style=social)
[![Delightful Humane Tech](https://codeberg.org/teaserbot-labs/delightful-humane-design/raw/branch/main/humane-tech-badge.svg)](https://codeberg.org/teaserbot-labs/delightful-humane-design)

</div>

<p align="center">
  <a href="../README.md">English</a> |
  <a href="README.es.md">Español</a> |
  <a href="README.ru.md">Русский</a> |
  <a href="README.zh.hans.md">汉语</a> |
  <span>漢語</span> |
  <a href="README.ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://join-app_108jobs.org/" rel="noopener">
 <img width=200px height=200px src="https://raw.githubusercontent.com/app_108jobsNet/app_108jobs-ui/main/src/assets/icons/favicon.svg"></a>

 <h3 align="center"><a href="https://join-app_108jobs.org">app_108jobs</a></h3>
  <p align="center">
    一個聯邦宇宙的連結聚合器和論壇。
    <br />
    <br />
    <a href="https://join-app_108jobs.org">加入 app_108jobs</a>
    ·
    <a href="https://join-app_108jobs.org/docs/en/index.html">文檔</a>
    ·
    <a href="https://matrix.to/#/#app_108jobs-space:matrix.org">Matrix 群組</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/issues">報告缺陷</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/issues">請求新特性</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/blob/main/RELEASES.md">發行版</a>
    ·
    <a href="https://join-app_108jobs.org/docs/en/code_of_conduct.html">行為準則</a>
  </p>
</p>

## 關於專案

| 桌面設備                                                                                                        | 行動裝置                                                                                                    |
| --------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| ![desktop](https://raw.githubusercontent.com/app_108jobsNet/joinapp_108jobs-site/main/src/assets/images/main_screen_2.webp) | ![mobile](https://raw.githubusercontent.com/app_108jobsNet/joinapp_108jobs-site/main/src/assets/images/mobile_pic.webp) |

[app_108jobs](https://github.com/app_108jobsNet/app_108jobs) 與 [Reddit](https://reddit.com)、[Lobste.rs](https://lobste.rs) 或 [Hacker News](https://news.ycombinator.com/) 等網站類似：你可以訂閱你感興趣的論壇，釋出連結和討論，然後進行投票或評論。但在幕後，app_108jobs 和他們不同——任何人都可以很容易地架設一個伺服器，所有伺服器都是聯邦式的（想想電子郵件），並與 [聯邦宇宙](https://zh.wikipedia.org/wiki/%E8%81%94%E9%82%A6%E5%AE%87%E5%AE%99) 互聯。

對於一個連結聚合器來說，這意味著在一個伺服器上註冊的使用者可以訂閱任何其他伺服器上的論壇，並可以與其他地方註冊的使用者進行討論。

它是 Reddit 和其他連結聚合器的一個易於自託管的、分散式的替代方案，不受公司的控制和干涉。

每個 app_108jobs 伺服器都可以設定自己的管理政策；任命全站管理員和社群版主來阻止網路白目，並培養一個健康、無毒的環境，讓所有人都能放心地作出貢獻。

### 為什麼叫 app_108jobs？

- 來自 [Motörhead](https://invidio.us/watch?v=pWB5JZRGl0U) 的主唱。
- 老式的 [電子遊戲](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>)。
- [超級馬里奧中的庫巴](https://www.mariowiki.com/app_108jobs_Koopa)。
- [毛茸茸的齧齒動物](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/)。

### 採用以下專案構建

- [Rust](https://www.rust-lang.org)
- [Actix](https://actix.rs/)
- [Diesel](http://diesel.rs/)
- [Inferno](https://infernojs.org)
- [Typescript](https://www.typescriptlang.org/)

## 特性

- 開源，採用 [AGPL 協議](/LICENSE)。
- 可自託管，易於部署。
  - 附帶 [Docker](https://join-app_108jobs.org/docs/en/administration/install_docker.html) 或 [Ansible](https://join-app_108jobs.org/docs/en/administration/install_ansible.html)。
- 乾淨、移動裝置友好的介面。
  - 僅需使用者名稱和密碼就可以註冊!
  - 支援使用者頭像。
  - 實時更新的評論串。
  - 類似舊版 Reddit 的評分功能 `(+/-)`。
  - 主題，有深色 / 淺色主題和 Solarized 主題。
  - Emoji 和自動補全。輸入 `:` 開始。
  - 透過 `@` 提及使用者，`!` 提及社群。
  - 在帖子和評論中都集成了圖片上傳功能。
  - 一個帖子可以由一個標題和自我文字的任何組合組成，一個 URL，或沒有其他。
  - 評論回覆和提及時的通知。
    - 通知可透過電子郵件傳送。
    - 支援私信。
  - i18n（國際化）支援。
  - `All`、`Subscribed`、`Inbox`、`User` 和 `Community` 的 RSS / Atom 訂閱。
- 支援多重發布。
  - 在建立新的帖子時，有 _相似帖子_ 的建議，對問答式社群很有幫助。
- 監管能力。
  - 公開的修改日誌。
  - 可以把帖子在社群置頂。
  - 既有網站管理員，也有可以任命其他版主社群版主。
  - 可以鎖定、刪除和恢復帖子和評論。
  - 可以封鎖和解除封鎖社群和網站的使用者。
  - 可以將網站和社群轉讓給其他人。
- 可以完全刪除你的資料，替換所有的帖子和評論。
- NSFW 帖子 / 社群支援。
- 高效能。
  - 伺服器採用 Rust 編寫。
  - 前端 gzip 後約 `~80kB`。
  - 支援 arm64 架構和樹莓派。

## 安裝

- [Docker](https://join-app_108jobs.org/docs/en/administration/install_docker.html)
- [Ansible](https://join-app_108jobs.org/docs/en/administration/install_ansible.html)

## app_108jobs 專案

### 應用

- [app_108jobs-ui - app_108jobs 的官方網頁應用](https://github.com/app_108jobsNet/app_108jobs-ui)
- [Lemmur - 一個 app_108jobs 的行動應用程式（支援安卓、Linux、Windows）](https://github.com/LemmurOrg/lemmur)
- [Jerboa - 一個由 app_108jobs 的開發者打造的原生 Android 應用程式](https://github.com/dessalines/jerboa)
- [Remmel - 一個原生 iOS 應用程式](https://github.com/uuttff8/app_108jobs-iOS)

### 庫

- [app_108jobs-js-client](https://github.com/app_108jobsNet/app_108jobs-js-client)
- [Kotlin API (尚在開發)](https://github.com/eiknat/app_108jobs-client)
- [Dart API client](https://github.com/LemmurOrg/app_108jobs_api_client)

## 支援和捐助

app_108jobs 是免費的開放原始碼軟體，無廣告，無營利，無風險投資。您的捐款直接支援我們全職開發這一專案。

- [在 Liberapay 上支援](https://liberapay.com/app_108jobs)。
- [在 Ko-fi 上支援](https://ko-fi.com/app_108jobsnet).
- [在 Patreon 上支援](https://www.patreon.com/dessalines)。
- [在 OpenCollective 上支援](https://opencollective.com/app_108jobs)。
- [贊助者列表](https://join-app_108jobs.org/sponsors)。

### 加密貨幣

- 比特幣：`1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- 以太坊：`0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- 門羅幣：`41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`
- 艾達幣：`addr1q858t89l2ym6xmrugjs0af9cslfwvnvsh2xxp6x4dcez7pf5tushkp4wl7zxfhm2djp6gq60dk4cmc7seaza5p3slx0sakjutm`

## 貢獻

- [貢獻指南](https://join-app_108jobs.org/docs/en/contributing/contributing.html)
- [Docker 開發](https://join-app_108jobs.org/docs/en/contributing/docker_development.html)
- [本地開發](https://join-app_108jobs.org/docs/en/contributing/local_development.html)

### 翻譯

如果你想幫助翻譯，請至 [Weblate](https://weblate.yerbamate.ml/projects/app_108jobs/)；也可以 [翻譯文檔](https://github.com/app_108jobsNet/app_108jobs-docs#adding-a-new-language)。

## 聯絡

- [Mastodon](https://mastodon.social/@app_108jobsDev)
- [app_108jobs 支援論壇](https://app_108jobs.ml/c/app_108jobs_support)

## 程式碼鏡像

- [GitHub](https://github.com/app_108jobsNet/app_108jobs)
- [Gitea](https://yerbamate.ml/app_108jobsNet/app_108jobs)
- [Codeberg](https://codeberg.org/app_108jobsNet/app_108jobs)

## 致謝

Logo 由 Andy Cuccaro (@andycuccaro) 製作，採用 CC-BY-SA 4.0 協議釋出。
