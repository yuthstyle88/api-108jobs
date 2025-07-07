use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use lemmy_api_utils::{
  context::FastJobContext,
};
use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::ExchangeKey;
use lemmy_utils::error::{ FastJobResult};

pub async fn exchange_keys(
  data: Json<ExchangeKey>,
  _req: HttpRequest,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ExchangeKey>> {
  let client_public_key_pem = hex::decode(data.public_key.clone())?;
  let sever = webcryptobox::generate_private_key()?;
  let sever_public_key = webcryptobox::get_public_key(&sever)?;
  let sever_public_key_pem = webcryptobox::export_public_key_pem(&sever_public_key)?;

  let pem_block = pem::parse(&sever_public_key_pem)?;
  let public_key_hex: SensitiveString = hex::encode(pem_block.contents()).into();
   let _client_key = webcryptobox::import_public_key_pem(&client_public_key_pem)?;

  let _ = LocalUser::update_public_key(&mut context.pool(), local_user_view.local_user.id, &data.public_key);
  let res = ExchangeKey{ public_key: public_key_hex };
  Ok(Json(res))
}
