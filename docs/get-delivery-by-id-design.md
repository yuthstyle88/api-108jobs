# GET /deliveries/{postId} — design

- Date: 2026-06-26
- Branch: `feat/get-delivery-by-id`
- Rider gap-closure (backend): add the single-delivery detail endpoint the
  Flutter client already calls.

## Problem

The Flutter client's `DeliveryApi.getDeliveryDetails` issues
`GET /api/v4/deliveries/{postId}`, but the backend `deliveries` scope only served
`/active`, `/completed`, `/cancelled`, `/{postId}/location`, `/status`,
`/assign`, `/confirm` — no plain detail-by-id. The client call 404s.

## Design

Thin handler mirroring the existing list handlers, reusing the DB method that
already exists:

- `DeliveryDetails::get_by_post_id(pool, post_id)` (db_schema, returns `Self` and
  maps a missing row to `FastJobErrorType::NotFound`) — already implemented.
- New `get_delivery(path: Path<PostId>, context)` in
  `crates/api/api/src/delivery/list.rs` → `Json<DeliveryDetails>`.
- Route `GET /deliveries/{postId}` registered in `src/api_routes.rs`.

Response shape is the **same `DeliveryDetails`** the list endpoints already
return (camelCase), so the client parses it with its existing model — no new
contract.

### Route ordering

`/{postId}` is a single-segment dynamic route registered **after** the literal
`/active`, `/completed`, `/cancelled` routes so those literals match first
(actix matches scope resources in registration order). It does not collide with
the deeper `/{postId}/location|status|assign|confirm` routes.

## Scope (Allowed Files)

- `crates/api/api/src/delivery/list.rs` (new `get_delivery` handler)
- `src/api_routes.rs` (import + route registration)
- `docs/get-delivery-by-id-design.md` (this doc)

Out of scope: auth tightening (matches the other delivery reads — authenticated
users), pagination, db_views changes.

## Verification

- Local: `cargo check` / `cargo clippy` compile clean (no live Postgres here).
- CI (woodpecker / GitHub Actions, which run a Postgres service):
  `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace`.

Note: this repo's delivery handlers are thin DB glue with no per-endpoint unit
tests (the prior contract-gap endpoints were added the same way); behaviour is
covered by the DB-backed workspace test suite in CI, which needs Postgres and
cannot run in this local environment.
