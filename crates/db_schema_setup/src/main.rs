/// Very minimal wrapper around `app_108jobs_db_schema_setup::run` to allow running migrations without
/// compiling everything.
fn main() -> anyhow::Result<()> {
  if std::env::args().len() > 1 {
    anyhow::bail!("To set parameters for running migrations, use the app_108jobs_server command.");
  }

  app_108jobs_db_schema_setup::run(
    app_108jobs_db_schema_setup::Options::default().run(),
    &std::env::var("app_108jobs_DATABASE_URL")?,
  )?;

  Ok(())
}
