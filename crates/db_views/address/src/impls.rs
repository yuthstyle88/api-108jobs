use crate::api::{UpsertAddress, UpsertAddressRequest};
use crate::AddressView;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::AddressId;
use lemmy_db_schema::utils::{get_conn, DbPool};
use lemmy_db_schema_file::schema::address;
use lemmy_utils::error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult};
use lemmy_utils::utils::validation::{get_required_trimmed};

impl AddressView {
  pub async fn find_by_id(
    pool: &mut DbPool<'_>,
    address_id: AddressId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    address::table
      .filter(address::id.eq(address_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntFindAddress)
  }
}

impl TryFrom<UpsertAddressRequest> for UpsertAddress {
  type Error = FastJobError;

  fn try_from(form: UpsertAddressRequest) -> Result<Self, Self::Error> {
    let address_line1 = get_required_trimmed(&form.address_line1, FastJobErrorType::EmptyAddressLine1)?;
    let subdistrict = get_required_trimmed(&form.subdistrict, FastJobErrorType::EmptySubdistrict)?;
    let district = get_required_trimmed(&form.district, FastJobErrorType::EmptyDistrict)?;
    let province = get_required_trimmed(&form.province, FastJobErrorType::EmptyProvince)?;
    let postal_code = get_required_trimmed(&form.postal_code, FastJobErrorType::EmptyPostalCode)?;

    let country_id = get_required_trimmed(&form.country_id, FastJobErrorType::EmptyCountryID)?;

    Ok(Self {
      address_line1,
      address_line2: form.address_line2,
      subdistrict,
      district,
      province,
      postal_code,
      country_id,
      is_default: form.is_default,
    })
  }
}