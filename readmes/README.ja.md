<div align="center">

![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/app_108jobsNet/app_108jobs.svg)
[![Build Status](https://woodpecker.join-app_108jobs.org/api/badges/app_108jobsNet/app_108jobs/status.svg)](https://woodpecker.join-app_108jobs.org/app_108jobsNet/app_108jobs)
[![GitHub issues](https://img.shields.io/github/issues-raw/app_108jobsNet/app_108jobs.svg)](https://github.com/app_108jobsNet/app_108jobs/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/app_108jobs.svg)](https://cloud.docker.com/repository/docker/dessalines/app_108jobs/)
[![Translation status](http://weblate.join-app_108jobs.org/widgets/app_108jobs/-/app_108jobs/svg-badge.svg)](http://weblate.join-app_108jobs.org/engage/app_108jobs/)
[![License](https://img.shields.io/github/license/app_108jobsNet/app_108jobs.svg)](LICENSE)
![GitHub stars](https://img.shields.io/github/stars/app_108jobsNet/app_108jobs?style=social)
[![Delightful Humane Tech](https://codeberg.org/teaserbot-labs/delightful-humane-design/raw/branch/main/humane-tech-badge.svg)](https://codeberg.org/teaserbot-labs/delightful-humane-design)

</div>

<p align="center">
  <a href="../README.md">English</a> |
  <a href="README.ru.md">Español</a> |
  <a href="README.ru.md">Русский</a> |
  <a href="README.zh.hans.md">汉语</a> |
  <a href="README.zh.hant.md">漢語</a> |
  <span>日本語</span>
</p>

<p align="center">
  <a href="https://join-app_108jobs.org/" rel="noopener">
 <img width=200px height=200px src="https://raw.githubusercontent.com/app_108jobsNet/app_108jobs-ui/main/src/assets/icons/favicon.svg"></a>

 <h3 align="center"><a href="https://join-app_108jobs.org">app_108jobs</a></h3>
  <p align="center">
    フェディバースのリンクアグリゲーターとフォーラムです。
    <br />
    <br />
    <a href="https://join-app_108jobs.org">app_108jobs に参加</a>
    ·
    <a href="https://join-app_108jobs.org/docs/en/index.html">ドキュメント</a>
    ·
    <a href="https://matrix.to/#/#app_108jobs-space:matrix.org">マトリックスチャット</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/issues">バグを報告</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/issues">機能リクエスト</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/blob/main/RELEASES.md">リリース</a>
    ·
    <a href="https://join-app_108jobs.org/docs/en/code_of_conduct.html">行動規範</a>
  </p>
</p>

## プロジェクトについて

| デスクトップ                                                                                                    | モバイル                                                                                                    |
| --------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| ![desktop](https://raw.githubusercontent.com/app_108jobsNet/joinapp_108jobs-site/main/src/assets/images/main_screen_2.webp) | ![mobile](https://raw.githubusercontent.com/app_108jobsNet/joinapp_108jobs-site/main/src/assets/images/mobile_pic.webp) |

[app_108jobs](https://github.com/app_108jobsNet/app_108jobs) は、[Reddit](https://reddit.com)、[Lobste.rs](https://lobste.rs)、[Hacker News](https://news.ycombinator.com/) といったサイトに似ています。興味のあるフォーラムを購読してリンクや議論を掲載し、投票したり、コメントしたりしています。誰でも簡単にサーバーを運営することができ、これらのサーバーはすべて連合しており（電子メールを考えてください）、[Fediverse](https://en.wikipedia.org/wiki/Fediverse) と呼ばれる同じ宇宙に接続されています。

リンクアグリゲーターの場合、あるサーバーに登録したユーザーが他のサーバーのフォーラムを購読し、他のサーバーに登録したユーザーとディスカッションができることを意味します。

Reddit や他のリンクアグリゲーターに代わる、企業の支配や干渉を受けない、簡単にセルフホスティングできる分散型の代替手段です。

各 app_108jobs サーバーは、独自のモデレーションポリシーを設定することができます。サイト全体の管理者やコミュニティモデレーターを任命し、荒らしを排除し、誰もが安心して貢献できる健全で毒気のない環境を育みます。

### なぜ app_108jobs というのか？

- [Motörhead](https://invidio.us/watch?v=3mbvWn1EY6g) のリードシンガー。
- 古くは[ビデオゲーム](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>)。
- [スーパーマリオのクッパ](https://www.mariowiki.com/app_108jobs_Koopa)。
- [毛むくじゃらの齧歯類](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/)。

### こちらでビルド

- [Rust](https://www.rust-lang.org)
- [Actix](https://actix.rs/)
- [Diesel](http://diesel.rs/)
- [Inferno](https://infernojs.org)
- [Typescript](https://www.typescriptlang.org/)

## 特徴

- オープンソース、[AGPL License](/LICENSE) です。
- セルフホスティングが可能で、デプロイが容易です。
  - [Docker](https://join-app_108jobs.org/docs/en/administration/install_docker.html) と [Ansible](https://join-app_108jobs.org/docs/en/administration/install_ansible.html) が付属しています。
- クリーンでモバイルフレンドリーなインターフェイス。
  - サインアップに必要なのは、最低限のユーザー名とパスワードのみ！
  - ユーザーアバター対応
  - ライブ更新のコメントスレッド
  - 古い Reddit のような完全な投票スコア `(+/-)`.
  - ライト、ダーク、ソラライズなどのテーマがあります。
  - オートコンプリートをサポートする絵文字。`:` と入力することでスタート
  - ユーザータグは `@` で、コミュニティタグは `!` で入力できます。
  - 投稿とコメントの両方で、画像のアップロードが可能です。
  - 投稿は、タイトルと自己テキスト、URL、またはそれ以外の任意の組み合わせで構成できます。
  - コメントの返信や、タグ付けされたときに、通知します。
    - 通知はメールで送ることができます。
    - プライベートメッセージのサポート
  - i18n / 国際化のサポート
  - `All`、`Subscribed`、`Inbox`、`User`、`Community` の RSS / Atom フィードを提供します。
- クロスポストのサポート。
  - 新しい投稿を作成する際の _類似投稿検索_。質問/回答コミュニティに最適です。
- モデレーション機能。
  - モデレーションのログを公開。
  - コミュニティのトップページにスティッキー・ポストを貼ることができます。
  - サイト管理者、コミュニティモデレーターの両方が、他のモデレーターを任命することができます。
  - 投稿やコメントのロック、削除、復元が可能。
  - コミュニティやサイトの利用を禁止したり、禁止を解除したりすることができます。
  - サイトとコミュニティを他者に譲渡することができます。
- すべての投稿とコメントを削除し、データを完全に消去することができます。
- NSFW 投稿/コミュニティサポート
- 高いパフォーマンス。
  - サーバーは Rust で書かれています。
  - フロントエンドは `~80kB` gzipped です。
  - arm64 / Raspberry Pi をサポートします。

## インストール

- [Docker](https://join-app_108jobs.org/docs/en/administration/install_docker.html)
- [Ansible](https://join-app_108jobs.org/docs/en/administration/install_ansible.html)

## app_108jobs プロジェクト

### アプリ

- [app_108jobs-ui - app_108jobs の公式ウェブアプリ](https://github.com/app_108jobsNet/app_108jobs-ui)
- [app_108jobsBB -phpBB をベースにした app_108jobs フォーラム UI](https://github.com/app_108jobsNet/app_108jobsBB)
- [Jerboa - app_108jobs の開発者が作った Android ネイティブアプリ](https://github.com/dessalines/jerboa)
- [Mlem - iOS 用 app_108jobs クライアント](https://github.com/buresdv/Mlem)

### ライブラリ

- [app_108jobs-js-client](https://github.com/app_108jobsNet/app_108jobs-js-client)
- [app_108jobs-rust-client](https://github.com/app_108jobsNet/app_108jobs/tree/main/crates/api_common)
- [go-app_108jobs](https://gitea.arsenm.dev/Arsen6331/go-app_108jobs)
- [Dart API client](https://github.com/LemmurOrg/app_108jobs_api_client)
- [app_108jobs-Swift-Client](https://github.com/rrainn/app_108jobs-Swift-Client)
- [Reddit -> app_108jobs Importer](https://github.com/rileynull/Redditapp_108jobsImporter)
- [app_108jobs-bot - app_108jobs のボットを簡単に作るための Typescript ライブラリ](https://github.com/SleeplessOne1917/app_108jobs-bot)
- [app_108jobs の Reddit API ラッパー](https://github.com/derivator/tafkars)
- [Pythörhead - app_108jobs API と統合するための Python パッケージ](https://pypi.org/project/pythorhead/)

## サポート / 寄付

app_108jobs はフリーでオープンソースのソフトウェアです。つまり、広告やマネタイズ、ベンチャーキャピタルは一切ありません。あなたの寄付は、直接プロジェクトのフルタイム開発をサポートします。

- [Liberapay でのサポート](https://liberapay.com/app_108jobs)。
- [Ko-fi でのサポート](https://ko-fi.com/app_108jobsnet).
- [Patreon でのサポート](https://www.patreon.com/dessalines)。
- [OpenCollective でのサポート](https://opencollective.com/app_108jobs)。
- [スポンサーのリスト](https://join-app_108jobs.org/donate)。

### 暗号通貨

- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## コントリビュート

- [コントリビュート手順](https://join-app_108jobs.org/docs/en/contributors/01-overview.html)
- [Docker 開発](https://join-app_108jobs.org/docs/en/contributors/03-docker-development.html)
- [Local 開発](https://join-app_108jobs.org/docs/en/contributors/02-local-development.html)

### 翻訳について

- 翻訳を手伝いたい方は、[Weblate](https://weblate.join-app_108jobs.org/projects/app_108jobs/) を見てみてください。また、[ドキュメントを翻訳する](https://github.com/app_108jobsNet/app_108jobs-docs#adding-a-new-language)ことでも支援できます。

## お問い合わせ

- [Mastodon](https://mastodon.social/@app_108jobsDev)
- [app_108jobs サポートフォーラム](https://app_108jobs.ml/c/app_108jobs_support)

## コードのミラー

- [GitHub](https://github.com/app_108jobsNet/app_108jobs)
- [Gitea](https://git.join-app_108jobs.org/app_108jobsNet/app_108jobs)
- [Codeberg](https://codeberg.org/app_108jobsNet/app_108jobs)

## クレジット

ロゴは Andy Cuccaro (@andycuccaro) が CC-BY-SA 4.0 ライセンスで作成しました。
