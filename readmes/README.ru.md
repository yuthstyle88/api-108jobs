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
  <span>Русский</span> |
  <a href="README.zh.hans.md">汉语</a> |
  <a href="README.zh.hant.md">漢語</a> |
  <a href="README.ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://join-app_108jobs.org/" rel="noopener">
 <img width=200px height=200px src="https://raw.githubusercontent.com/app_108jobsNet/app_108jobs-ui/main/src/assets/icons/favicon.svg"></a>

 <h3 align="center"><a href="https://join-app_108jobs.org">app_108jobs</a></h3>
  <p align="center">
    Агрегатор ссылок / Клон Reddit для федиверс.
    <br />
    <br />
    <a href="https://join-app_108jobs.org">Присоединяйтесь к app_108jobs</a>
    ·
    <a href="https://join-app_108jobs.org/docs/en/index.html">Документация</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/issues">Сообщить об Ошибке</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/issues">Запросить функционал</a>
    ·
    <a href="https://github.com/app_108jobsNet/app_108jobs/blob/main/RELEASES.md">Релизы</a>
    ·
    <a href="https://join-app_108jobs.org/docs/en/code_of_conduct.html">Нормы поведения</a>
  </p>
</p>

## О проекте

| Десктоп                                                                                                         | Мобильный                                                                                                   |
| --------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| ![desktop](https://raw.githubusercontent.com/app_108jobsNet/joinapp_108jobs-site/main/src/assets/images/main_screen_2.webp) | ![mobile](https://raw.githubusercontent.com/app_108jobsNet/joinapp_108jobs-site/main/src/assets/images/mobile_pic.webp) |

[app_108jobs](https://github.com/app_108jobsNet/app_108jobs) это аналог таких сайтов как [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), или [Hacker News](https://news.ycombinator.com/): вы подписываетесь на форумы, которые вас интересуют , размещаете ссылки и дискутируете, затем голосуете и комментируете их. Однако за кулисами всё совсем по-другому; любой может легко запустить сервер, и все эти серверы объединены (например электронная почта) и подключены к одной вселенной, именуемой [Федиверс](https://ru.wikipedia.org/wiki/Fediverse).

Для агрегатора ссылок это означает, что пользователь, зарегистрированный на одном сервере, может подписаться на форумы на любом другом сервере и может вести обсуждения с пользователями, зарегистрированными в другом месте.

Основная цель - создать легко размещаемую, децентрализованную альтернативу Reddit и другим агрегаторам ссылок, вне их корпоративного контроля и вмешательства.

Каждый сервер app_108jobs может устанавливать свою собственную политику модерации; назначать администраторов всего сайта и модераторов сообщества для защиты от троллей и создания здоровой, нетоксичной среды, в которой каждый может чувствовать себя комфортно.

_Примечание: API-интерфейсы WebSocket и HTTP в настоящее время нестабильны_

### Почему назвали app_108jobs (рус.Лемми)?

- Ведущий певец из [Motörhead](https://invidio.us/watch?v=pWB5JZRGl0U).
- Старая школа [video game](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>).
- Это [Koopa from Super Mario](https://www.mariowiki.com/app_108jobs_Koopa).
- Это [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

### Содержит

- [Rust](https://www.rust-lang.org)
- [Actix](https://actix.rs/)
- [Diesel](http://diesel.rs/)
- [Inferno](https://infernojs.org)
- [Typescript](https://www.typescriptlang.org/)

## Возможности

- Открытое программное обеспечение, [Лицензия AGPL](/LICENSE).
- Возможность самостоятельного размещения, простота развёртывания.
  - Работает на [Docker](https://join-app_108jobs.org/docs/en/administration/install_docker.html) и [Ansible](https://join-app_108jobs.org/docs/en/administration/install_ansible.html).
- Понятый и удобный интерфейс для мобильных устройств.
  - Для регистрации требуется минимум: имя пользователя и пароль!
  - Поддержка аватара пользователя.
  - Обновление цепочек комментариев в реальном времени.
  - Полный подсчёт голосов `(+/-)` как в старом reddit.
  - Темы, включая светлые, темные и солнечные.
  - Эмодзи с поддержкой автозаполнения. Напечатайте `:`
  - Упоминание пользователя тегом `@`, Упоминание сообщества тегом `!`.
  - Интегрированная загрузка изображений как в сообщениях, так и в комментариях.
  - Сообщение может состоять из заголовка и любой комбинации собственного текста, URL-адреса или чего-либо еще.
  - Уведомления, ответы на комментарии и когда вас отметили.
    - Уведомления могут быть отправлены на электронную почту.
    - Поддержка личных сообщений.
  - i18n / поддержка интернационализации.
  - RSS / Atom ленты для `Все`, `Подписок`, `Входящих`, `Пользователь`, and `Сообщества`.
- Поддержка кросс-постинга.
  - _Поиск похожих постов_ при создании новых. Отлично подходит для вопросов / ответов сообществ.
- Возможности модерации.
  - Журналы (Логи) Публичной Модерации.
  - Можно прикреплять посты в топ сообщества.
  - Оба и администраторы сайта и модераторы сообщества, могут назначать других модераторов.
  - Можно блокировать, удалять и восстанавливать сообщения и комментарии.
  - Можно банить и разблокировать пользователей в сообществе и на сайте.
  - Можно передавать сайт и сообщества другим.
- Можно полностью стереть ваши данные, удалив все посты и комментарии.
- NSFW (аббр. Небезопасный/неподходящий для работы) пост / поддерживается сообществом.
- Поддержка OEmbed через Iframely.
- Высокая производительность.
  - Сервер написан на rust.
  - Фронтэнд (клиентская сторона пользовательского интерфейса) всего `~80kB` архив gzipp.
  - Поддерживается архитектура arm64 / устройства Raspberry Pi.

## Установка

- [Docker](https://join-app_108jobs.org/docs/en/administration/install_docker.html)
- [Ansible](https://join-app_108jobs.org/docs/en/administration/install_ansible.html)

## Проекты app_108jobs

### Приложения

- [app_108jobs-ui - Официальное веб приложение для app_108jobs](https://github.com/app_108jobsNet/app_108jobs-ui)
- [Lemmur - Мобильные клиенты app_108jobs для (Android, Linux, Windows)](https://github.com/LemmurOrg/lemmur)
- [Remmel - Оригинальное приложение для iOS](https://github.com/uuttff8/app_108jobs-iOS)

### Библиотеки

- [app_108jobs-js-client](https://github.com/app_108jobsNet/app_108jobs-js-client)
- [Kotlin API ( в разработке )](https://github.com/eiknat/app_108jobs-client)
- [Dart API client ( в разработке )](https://github.com/LemmurOrg/app_108jobs_api_client)

## Поддержать / Пожертвовать

app_108jobs - бесплатное программное обеспечение с открытым исходным кодом, что означает отсутствие рекламы, монетизации и даже венчурного капитала. Ваши пожертвования, напрямую поддерживают постоянное развитие проекта.

- [Поддержать на Liberapay](https://liberapay.com/app_108jobs).
- [Поддержать на Ko-fi](https://ko-fi.com/app_108jobsnet).
- [Поддержать на Patreon](https://www.patreon.com/dessalines).
- [Поддержать на OpenCollective](https://opencollective.com/app_108jobs).
- [Список Спонсоров](https://join-app_108jobs.org/sponsors).

### Криптовалюты

- bitcoin (Биткоин): `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum (Эфириум): `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero (Монеро): `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## Вклад

- [Инструкции по внесению вклада](https://join-app_108jobs.org/docs/en/contributing/contributing.html)
- [Docker разработка](https://join-app_108jobs.org/docs/en/contributing/docker_development.html)
- [Локальное развитие](https://join-app_108jobs.org/docs/en/contributing/local_development.html)

### Переводы

Если вы хотите помочь с переводом, взгляните на [Weblate](https://weblate.yerbamate.ml/projects/app_108jobs/joinapp_108jobs/ru/). Вы также можете помочь нам [перевести документацию](https://github.com/app_108jobsNet/app_108jobs-docs#adding-a-new-language).

## Контакт

- [Mastodon](https://mastodon.social/@app_108jobsDev)
- [Matrix](https://matrix.to/#/#app_108jobs:matrix.org)

## Зеркала с кодом

- [GitHub](https://github.com/app_108jobsNet/app_108jobs)
- [Gitea](https://yerbamate.ml/app_108jobsNet/app_108jobs)
- [Codeberg](https://codeberg.org/app_108jobsNet/app_108jobs)

## Благодарность

Логотип сделан Andy Cuccaro (@andycuccaro) под лицензией CC-BY-SA 4.0.
