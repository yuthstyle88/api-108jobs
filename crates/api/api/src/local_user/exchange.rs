use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use lemmy_api_utils::{
  context::FastJobContext,
};

use lemmy_db_views_site::api::ExchangeKey;
use lemmy_utils::error::{ FastJobResult};

pub async fn exchange_keys(
  data: Json<ExchangeKey>,
  _req: HttpRequest,
  _context: Data<FastJobContext>,
) -> FastJobResult<Json<ExchangeKey>> {
  let client_public_key_pem = hex::decode(data.public_key.clone())?;
  let sever = webcryptobox::generate_private_key()?;
  let sever_public_key = webcryptobox::get_public_key(&sever)?;
  let sever_public_key_pem = webcryptobox::export_public_key_pem(&sever_public_key)?;

  let pem_block = pem::parse(&sever_public_key_pem)?;
  let public_key_hex = hex::encode(pem_block.contents());
   let client_key = webcryptobox::import_public_key_pem(&client_public_key_pem)?;
  //Todo save client key to DB
  let res = ExchangeKey{ public_key: public_key_hex };
 Ok(Json(res))
}
