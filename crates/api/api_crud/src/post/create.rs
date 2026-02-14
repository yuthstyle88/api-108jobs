use super::convert_published_time;
use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::utils::check_category_deleted_removed;
use app_108jobs_api_utils::{
  build_response::{build_post_response, send_local_notifs},
  context::FastJobContext,
  tags::update_post_tags,
  utils::{
    check_self_promotion_allowed, get_url_blocklist, honeypot_check,
    process_markdown_opt, slur_regex,
  },
};
use app_108jobs_db_schema::source::delivery_details::{DeliveryDetails, DeliveryDetailsInsertForm};
use app_108jobs_db_schema::source::ride_session::RideSessionInsertForm;
use app_108jobs_db_schema::{
  impls::actor_language::{validate_post_language, UNDETERMINED_ID},
  source::post::{Post, PostActions, PostInsertForm, PostLikeForm, PostReadForm},
  traits::{Crud, Likeable, Readable},
  utils::diesel_url_create,
};
use app_108jobs_db_schema_file::enums::{DeliveryStatus, PostKind};
use app_108jobs_db_views_category::CategoryView;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::api::{CreatePost, CreatePostRequest, PostResponse};
use app_108jobs_utils::error::FastJobErrorExt2;
use app_108jobs_utils::error::FastJobErrorType;
use app_108jobs_utils::{
  error::FastJobResult,
  utils::{
    slurs::check_slurs,
    validation::{is_url_blocked, is_valid_alt_text_field, is_valid_body_field, is_valid_url},
  },
};

pub async fn create_post(
  data: Json<CreatePostRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostResponse>> {
  Post::check_post_name_taken(&mut context.pool(), &data.name).await?;

  let data: CreatePost = data.into_inner().try_into()?;
  honeypot_check(&data.honeypot)?;
  let local_site= context.site_config().get().await?.site_view.local_site;

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&data.name, &slur_regex)?;
  let url_blocklist = get_url_blocklist(&context).await?;

  let body = process_markdown_opt(&data.body, &slur_regex, &url_blocklist, &context).await?;
  let url = diesel_url_create(data.url.as_deref())?;
  let custom_thumbnail = diesel_url_create(data.custom_thumbnail.as_deref())?;
  check_self_promotion_allowed(data.self_promotion, Some(&local_site))?;

  if let Some(url) = &url {
    is_url_blocked(url, &url_blocklist)?;
    is_valid_url(url)?;
  }

  if let Some(custom_thumbnail) = &custom_thumbnail {
    is_valid_url(custom_thumbnail)?;
  }

  if let Some(alt_text) = &data.alt_text {
    is_valid_alt_text_field(alt_text)?;
  }

  if let Some(body) = &body {
    is_valid_body_field(body, true)?;
  }

  // For posts with a category (normal posts, or delivery posts with explicit category)
  let (category_view, self_promotion, language_id) = if let Some(category_id) = data.category_id {
    let category_view = CategoryView::read(
      &mut context.pool(),
      category_id,
      Some(&local_user_view.local_user),
    )
    .await?;
    let category = &category_view.category;
    check_category_deleted_removed(&category)?;

    // Ensure that all posts in NSFW communities are marked as NSFW
    let self_promotion = if category.self_promotion {
      Some(true)
    } else {
      data.self_promotion
    };

    let language_id = validate_post_language(
      &mut context.pool(),
      data.language_id,
      category_id,
      local_user_view.local_user.id,
    )
    .await?;

    (Some(category_view), self_promotion, language_id)
  } else {
    // For delivery posts without a category, use default values
    (None, data.self_promotion, data.language_id.unwrap_or(UNDETERMINED_ID))
  };

  let scheduled_publish_time_at =
    convert_published_time(data.scheduled_publish_time_at, &local_user_view, &context).await?;
  let post_form = PostInsertForm {
    url,
    body,
    alt_text: data.alt_text.clone(),
    self_promotion,
    language_id: Some(language_id),
    scheduled_publish_time_at,
    budget: data.budget,
    ap_id: data.ap_id,
    job_type: data.job_type,
    intended_use: data.intended_use,
    deadline: data.deadline,
    is_english_required: data.is_english_required,
    post_kind: Some(data.post_kind),
    category_id: data.category_id,
    ..PostInsertForm::new(
      data.name.trim().to_string(),
      local_user_view.person.id,
    )
  };

  let inserted_post = Post::create(&mut context.pool(), &post_form).await?;

  // Persist logistics child based on post_kind
  if data.post_kind == PostKind::Delivery {
    let dd = data
      .delivery_details
      .as_ref()
      .ok_or(FastJobErrorType::InvalidField(
        "delivery_details required for Delivery post".to_string(),
      ))?;

    // Build insert form using struct literal; derive_new::new() only accepts required fields
    let dd_form = DeliveryDetailsInsertForm {
      post_id: inserted_post.id,
      pickup_address: dd.pickup_address.clone().ok_or(FastJobErrorType::InvalidField(
        "pickup_address is required".to_string(),
      ))?,
      pickup_lat: dd.pickup_lat,
      pickup_lng: dd.pickup_lng,
      dropoff_address: dd.dropoff_address.clone().ok_or(FastJobErrorType::InvalidField(
        "dropoff_address is required".to_string(),
      ))?,
      dropoff_lat: dd.dropoff_lat,
      dropoff_lng: dd.dropoff_lng,
      package_description: dd.package_description.clone(),
      package_weight_kg: dd.package_weight_kg,
      package_size: dd.package_size.clone(),
      fragile: dd.fragile,
      requires_signature: dd.requires_signature,
      vehicle_required: dd.vehicle_required,
      latest_pickup_at: dd.latest_pickup_at,
      latest_dropoff_at: dd.latest_dropoff_at,
      sender_name: dd.sender_name.clone(),
      sender_phone: dd.sender_phone.clone(),
      receiver_name: dd.receiver_name.clone(),
      receiver_phone: dd.receiver_phone.clone(),
      cash_on_delivery: dd.cash_on_delivery,
      cod_amount: dd.cod_amount,
      status: Some(DeliveryStatus::Pending),
      // Payment tracking fields (not set during post creation)
      delivery_fee: None,
      employer_confirmed_at: None,
      employer_wallet_transaction_id: None,
      rider_wallet_transaction_id: None,
    };

    DeliveryDetails::create(&mut context.pool(), &dd_form)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)?;
  } else if data.post_kind == PostKind::RideTaxi {
    let rp = data
      .ride_payload
      .as_ref()
      .ok_or(FastJobErrorType::InvalidField(
        "ride_payload required for RideTaxi post".to_string(),
      ))?;

    let session_form = RideSessionInsertForm {
      post_id: inserted_post.id,
      rider_id: None,
      employer_id: local_user_view.local_user.id,
      pricing_config_id: None,
      pickup_address: rp.pickup_address.clone(),
      pickup_lat: rp.pickup_lat,
      pickup_lng: rp.pickup_lng,
      dropoff_address: rp.dropoff_address.clone(),
      dropoff_lat: rp.dropoff_lat,
      dropoff_lng: rp.dropoff_lng,
      pickup_note: rp.pickup_note.clone(),
      payment_method: rp.payment_method,
      payment_status: Some("Pending".to_string()),
      status: Some(DeliveryStatus::Pending),
      requested_at: Some(chrono::Utc::now()),
      current_price_coin: Some(0),
    };

    app_108jobs_db_schema::source::ride_session::RideSession::create(
      &mut context.pool(),
      &session_form,
    )
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)?;
  }

  // Tags are only supported for posts with a category
  if let (Some(tags), Some(category_view)) = (&data.tags, category_view.as_ref()) {
    update_post_tags(
      &context,
      &inserted_post,
      category_view,
      tags,
      &local_user_view,
    )
    .await?;
  }

  // They like their own post by default
  let person_id = local_user_view.person.id;
  let post_id = inserted_post.id;
  let like_form = PostLikeForm::new(post_id, person_id, 1);

  PostActions::like(&mut context.pool(), &like_form).await?;

  send_local_notifs(
    &inserted_post,
    None,
    &local_user_view.person,
    &context,
  )
  .await?;

  let read_form = PostReadForm::new(post_id, person_id);
  PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

  build_post_response(&context, local_user_view, post_id).await
}
