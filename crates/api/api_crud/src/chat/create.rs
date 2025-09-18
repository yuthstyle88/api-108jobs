use actix_web::web::{Data, Json};
use chrono::Utc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_schema::source::chat_message::ChatMessage;
use lemmy_db_schema::source::chat_participant::{ChatParticipant, ChatParticipantInsertForm};
use lemmy_db_schema::source::chat_room::{ChatRoom, ChatRoomInsertForm, ChatRoomUpdateForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_chat::api::{ChatRoomResponse, CreateChatRoomRequest, LastMessage};
use lemmy_db_views_chat::ChatRoomView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;


/// POST /api/v4/chat/rooms
/// Create (or get) a direct-message chat room for two users, and ensure both are participants.
pub async fn create_chat_room(
  data: Json<CreateChatRoomRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ChatRoomResponse>> {
  let mut pool = context.pool();
  let req = data.into_inner();
  let CreateChatRoomRequest { partner_person_id, room_id, post_id, current_comment_id } = req;

  // current and partner local user ids
  let current_luid = local_user_view.local_user.id;
  let partner_luv = LocalUserView::read_person(&mut pool, partner_person_id).await?;
  let partner_luid = partner_luv.local_user.id;

  // resolve room id: prefer provided room_id, otherwise build from current then partner
  let room_id = room_id.unwrap_or_else(|| ChatRoomId(format!("dm:{}:{}", current_luid.0, partner_luid.0)));
  let room_name = room_id.0.clone();

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
      room_name: None,
      updated_at: Some(Utc::now()),
      post_id: post_id.clone().map(Some),
      current_comment_id: current_comment_id.clone().map(Some),
    };
    // Only call update if at least one optional is provided
    if upd.post_id.is_some() || upd.current_comment_id.is_some() {
      let _ = ChatRoom::update(&mut pool, room_id.clone(), &upd).await?;
    }
  }

  // ensure participants: current user and partner
  let form1 = ChatParticipantInsertForm { room_id: room_id.clone(), member_id: current_luid };
  ChatParticipant::ensure_participant(&mut pool, &form1).await?;
  if current_luid != partner_luid {
    let form2 = ChatParticipantInsertForm { room_id: room_id.clone(), member_id: partner_luid };
    ChatParticipant::ensure_participant(&mut pool, &form2).await?;
  }

  // build full view
  let view = ChatRoomView::read(&mut pool, room_id.clone()).await?;

  // last message
  let last_message_opt = ChatMessage::last_by_room(&mut pool, room_id).await?;
  let last_message = last_message_opt.map(|m| LastMessage {
    content: m.content,
    timestamp: m.created_at.to_rfc3339(),
    sender_id: m.sender_id,
  });

  Ok(Json(ChatRoomResponse { room: view, last_message, workflow: None }))
}
