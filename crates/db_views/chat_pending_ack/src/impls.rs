use crate::{AckConfirmRequest, AckConfirmResponse, AckReminderQuery, AckReminderResponse, ChatPendingAckItem, ListChatPendingAckQuery, ListChatPendingAckResponse};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
    newtypes::{ChatRoomId, LocalUserId},
    utils::{get_conn, DbPool},
};
use lemmy_utils::error::FastJobResult;

use chrono::{DateTime, Utc};
use lemmy_db_schema_file::schema::pending_sender_ack;
use uuid::Uuid;
use lemmy_db_schema::source::pending_sender_ack::{PendingSenderAck, PendingSenderAckInsertForm};
use lemmy_db_schema::traits::Crud;

/// Internal: get a connection from pool

/// List pending ACK items for a stream.
pub async fn list_pending_ack(
    pool: &mut DbPool<'_>,
    q: &ListChatPendingAckQuery,
) -> FastJobResult<ListChatPendingAckResponse> {
    use pending_sender_ack::dsl as psa;

    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let conn = &mut get_conn(pool).await?;

    // NOTE: Pagination by `before` is optional and may depend on your PaginationCursor.
    // For now we ignore `before` to keep semantics simple and correct.
    let rows = psa::pending_sender_ack
        .filter(psa::room_id.eq(q.room_id.clone()))
        .filter(psa::sender_id.eq(q.sender_id))
        .order(psa::created_at.asc())
        .limit(limit)
        .select((
            psa::room_id,
            psa::sender_id,
            psa::client_id,
            psa::created_at,
        ))
        .load::<(String, i32, Uuid, DateTime<Utc>)>(conn)
        .await?;

    let items = rows
        .into_iter()
        .map(|(room_id, sender_id, client_id, created_at)| ChatPendingAckItem {
            room_id: ChatRoomId(room_id.into()),
            sender_id: LocalUserId(sender_id),
            client_id,
            created_at,
        })
        .collect();

    Ok(ListChatPendingAckResponse { items })
}

/// Enqueue a pending sender ack entry. Idempotent: ON CONFLICT DO NOTHING.
pub async fn enqueue_pending(
    pool: &mut DbPool<'_>,
    room_id: ChatRoomId,
    sender_id: Option<LocalUserId>,
    client_id: Option<Uuid>,
) -> FastJobResult<PendingSenderAck> {
    let pending_insert_form = PendingSenderAckInsertForm{
        room_id,
        sender_id,
        client_id,
        created_at: Default::default(),
    } ;
    let affected =   PendingSenderAck::create(pool, &pending_insert_form).await?;
    Ok(affected)
}

/// Get reminder list of clientIds that still need ackConfirm for this stream.
pub async fn ack_reminder(
    pool: &mut DbPool<'_>,
    q: &AckReminderQuery,
) -> FastJobResult<AckReminderResponse> {
    use pending_sender_ack::dsl as psa;

    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let conn = &mut get_conn(pool).await?;

    let ids = psa::pending_sender_ack
        .filter(psa::room_id.eq(q.room_id.clone()))
        .filter(psa::sender_id.eq(q.sender_id))
        .order(psa::created_at.asc())
        .limit(limit)
        .select(psa::client_id)
        .load::<Uuid>(conn)
        .await?;

    Ok(AckReminderResponse { client_ids: ids })
}

/// Confirm (ack-of-ack) a batch by clientIds. Idempotent: returns how many were removed.
pub async fn ack_confirm(
    pool: &mut DbPool<'_>,
    req: &AckConfirmRequest,
) -> FastJobResult<AckConfirmResponse> {
    use pending_sender_ack::dsl as psa;

    let conn = &mut get_conn(pool).await?;
    let mut removed: usize = 0;
    let mut not_found: Vec<Uuid> = Vec::new();

    // Simple, robust approach: delete each id individually so we can compute not_found.
    for cid in &req.client_ids {
        let n = diesel::delete(
            psa::pending_sender_ack
                .filter(psa::room_id.eq(req.room_id.clone()))
                .filter(psa::sender_id.eq(req.sender_id))
                .filter(psa::client_id.eq(*cid)),
        )
        .execute(conn)
        .await?;

        if n == 0 {
            not_found.push(*cid);
        } else {
            removed += n;
        }
    }

    Ok(AckConfirmResponse { removed, not_found })
}
