use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::newtypes::ChatRoomId;
use app_108jobs_db_schema::source::chat_participant::{ChatParticipant, ChatParticipantInsertForm};
use app_108jobs_db_schema::source::chat_room::{ChatRoom, ChatRoomInsertForm, ChatRoomUpdateForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_views_chat::api::{ChatRoomResponse, CreateChatRoomRequest};
use app_108jobs_db_views_chat::ChatRoomView;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;
use app_108jobs_utils::utils::keys::contacts_key;
use chrono::Utc;

/// POST /api/v4/chat/rooms
/// Create (or get) a direct-message chat room for two users, and ensure both are participants.
pub async fn create_chat_room(
  data: Json<CreateChatRoomRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ChatRoomResponse>> {
  let mut pool = context.pool();
  let req = data.into_inner();
  let CreateChatRoomRequest {
    partner_person_id,
    room_id,
    post_id,
    current_comment_id,
    room_name,
  } = req;

  // current and partner local user ids
  let current_luid = local_user_view.local_user.id;
  let partner_luv = LocalUserView::read_person(&mut pool, partner_person_id).await?;
  let partner_luid = partner_luv.local_user.id;

  // resolve room id: prefer provided room_id, otherwise build from current then partner
  let room_id =
    room_id.unwrap_or_else(|| ChatRoomId(format!("dm:{}:{}", current_luid.0, partner_luid.0)));

  // resolve room name: prefer provided, otherwise fallback to room_id string
  let room_name = room_name.unwrap_or_else(|| room_id.0.clone());

  // create room if not exists
  if !ChatRoom::exists(&mut pool, room_id.clone()).await? {
    let form = ChatRoomInsertForm {
      id: room_id.clone(),
      room_name: room_name.clone(),
      created_at: Utc::now(),
      updated_at: None,
      post_id: post_id.clone(),
      current_comment_id: current_comment_id.clone(),
    };
    let _ = ChatRoom::create(&mut pool, &form).await?;
  } else {
    // update existing room with any provided optional fields
    let upd = ChatRoomUpdateForm {
      room_name: room_name.clone().into(),
      updated_at: Some(Utc::now()),
      post_id: post_id.clone().map(Some),
      current_comment_id: current_comment_id.clone().map(Some),
      ..Default::default()
    };
    // Only call update if at least one optional is provided
    if upd.room_name.is_some() || upd.post_id.is_some() || upd.current_comment_id.is_some() {
      let _ = ChatRoom::update(&mut pool, room_id.clone(), &upd).await?;
    }
  }

  // ensure participants: current user and partner
  let form1 = ChatParticipantInsertForm {
    room_id: room_id.clone(),
    member_id: current_luid,
  };
  ChatParticipant::ensure_participant(&mut pool, &form1).await?;
  if current_luid != partner_luid {
    let form2 = ChatParticipantInsertForm {
      room_id: room_id.clone(),
      member_id: partner_luid,
    };
    ChatParticipant::ensure_participant(&mut pool, &form2).await?;
  }

  // --- Update Redis contacts ---
  let mut redis = context.redis().clone();

  let current_uid = current_luid.0;
  let partner_uid = partner_luid.0;

  // current user sees partner
  let _ = redis.sadd(&contacts_key(current_uid), partner_uid).await;

  // partner sees current user
  let _ = redis.sadd(&contacts_key(partner_uid), current_uid).await;

  let view = ChatRoomView::read(&mut pool, room_id.clone()).await?;

  Ok(Json(ChatRoomResponse { room: view }))
}
