use crate::api::{UpsertIDCard, UpsertIDCardRequest};
use crate::IdentityCardView;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::IdentityCardId;
use lemmy_db_schema::utils::{get_conn, DbPool};
use lemmy_db_schema_file::schema::identity_card;
use lemmy_utils::error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult};
use lemmy_utils::utils::validation::{
  get_required_trimmed, NationalIdValidator, ThaiIdValidator, VietnamIdValidator,
};

impl IdentityCardView {
  pub async fn find_by_id(
    pool: &mut DbPool<'_>,
    identity_card_id: IdentityCardId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    identity_card::table
      .filter(identity_card::id.eq(identity_card_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindIdentityCard)
  }
}

impl TryFrom<UpsertIDCardRequest> for UpsertIDCard {
  type Error = FastJobError;
  fn try_from(data: UpsertIDCardRequest) -> Result<Self, Self::Error> {
    let id_number = get_required_trimmed(&data.id_number, FastJobErrorType::EmptyIDNumber)?;
    let nationality = get_required_trimmed(&data.nationality, FastJobErrorType::EmptyNationality)?;
    let full_name = get_required_trimmed(&data.full_name, FastJobErrorType::EmpltyFullName)?;

    let is_valid = match nationality.to_lowercase().as_str() {
      "thailand" => ThaiIdValidator.is_valid(&id_number),
      "vietnam" => VietnamIdValidator.is_valid(&id_number),
      _ => true,
    };

    if !is_valid {
      return Err(FastJobErrorType::InvalidIDNumber.into());
    }

    let issued_date = data.issued_date.unwrap_or_default();
    let expiry_date = data.expiry_date.unwrap_or_default();
    let date_of_birth = data.date_of_birth.unwrap_or_default();

    Ok(UpsertIDCard {
      address_id: data.address_id.unwrap(),
      id_number,
      issued_date,
      expiry_date,
      full_name,
      date_of_birth,
      nationality,
    })
  }
}
