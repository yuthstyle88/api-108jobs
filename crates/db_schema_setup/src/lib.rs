mod diff_check;
use anyhow::{anyhow, Context};
use chrono::TimeDelta;
use diesel::{
  connection::SimpleConnection,
  dsl::exists,
  migration::{Migration, MigrationVersion},
  pg::Pg,
  select,
  update,
  BoolExpressionMethods,
  Connection,
  ExpressionMethods,
  PgConnection,
  QueryDsl,
  RunQueryDsl,
};
use diesel_migrations::MigrationHarness;
use std::time::Instant;
use tracing::debug;

diesel::table! {
  pg_namespace (nspname) {
    nspname -> Text,
  }
}

diesel::table! {
  previously_run_sql (id) {
    id -> Bool,
    content -> Text,
  }
}

fn migrations() -> diesel_migrations::EmbeddedMigrations {
  // Using `const` here is required by the borrow checker
  const MIGRATIONS: diesel_migrations::EmbeddedMigrations = diesel_migrations::embed_migrations!();
  MIGRATIONS
}

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and
/// replaced instead of being changed using migrations. It may not create or modify things outside
/// of the `r` schema (indicated by `r.` before the name), unless a comment says otherwise.
fn replaceable_schema() -> String {
  [
    "CREATE SCHEMA r;",
    include_str!("../replaceable_schema/utils.sql"),
    include_str!("../replaceable_schema/triggers.sql"),
  ]
  .join("\n")
}

const REPLACEABLE_SCHEMA_PATH: &str = "crates/db_schema_setup/replaceable_schema";

struct MigrationHarnessWrapper<'a> {
  conn: &'a mut PgConnection,
  #[cfg(test)]
  enable_diff_check: bool,
  options: &'a Options,
}

impl MigrationHarnessWrapper<'_> {
  fn run_migration_inner(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    let start_time = Instant::now();

    let result = self.conn.run_migration(migration);

    let duration = TimeDelta::from_std(start_time.elapsed())
      .map(|d| d.to_string())
      .unwrap_or_default();
    let name = migration.name();
    self.options.print(&format!("{duration} run {name}"));

    result
  }
}

impl MigrationHarness<Pg> for MigrationHarnessWrapper<'_> {
  fn run_migration(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    #[cfg(test)]
    if self.enable_diff_check {
      let before = diff_check::get_dump();

      self.run_migration_inner(migration)?;
      self.revert_migration(migration)?;

      let after = diff_check::get_dump();

      diff_check::check_dump_diff(
        [&after, &before],
        &format!(
          "These changes need to be applied in migrations/{}/down.sql:",
          migration.name()
        ),
      );
    }

    self.run_migration_inner(migration)
  }

  fn revert_migration(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    let start_time = Instant::now();

    let result = self.conn.revert_migration(migration);

    let duration = TimeDelta::from_std(start_time.elapsed())
      .map(|d| d.to_string())
      .unwrap_or_default();
    let name = migration.name();
    self.options.print(&format!("{duration} revert {name}"));

    result
  }

  fn applied_migrations(&mut self) -> diesel::migration::Result<Vec<MigrationVersion<'static>>> {
    self.conn.applied_migrations()
  }
}

#[derive(Default, Clone, Copy)]
pub struct Options {
  #[cfg(test)]
  enable_diff_check: bool,
  revert: bool,
  run: bool,
  print_output: bool,
  limit: Option<u64>,
}

impl Options {
  #[cfg(test)]
  fn enable_diff_check(mut self) -> Self {
    self.enable_diff_check = true;
    self
  }

  pub fn run(mut self) -> Self {
    self.run = true;
    self
  }

  pub fn revert(mut self) -> Self {
    self.revert = true;
    self
  }

  pub fn limit(mut self, limit: u64) -> Self {
    self.limit = Some(limit);
    self
  }

  /// If print_output is true, use println!.
  /// Otherwise, use debug!
  pub fn print_output(mut self) -> Self {
    self.print_output = true;
    self
  }

  fn print(&self, text: &str) {
    if self.print_output {
      println!("{text}");
    } else {
      debug!("{text}");
    }
  }
}

/// Checked by tests
#[derive(PartialEq, Eq, Debug)]
pub enum Branch {
  EarlyReturn,
  ReplaceableSchemaRebuilt,
  ReplaceableSchemaNotRebuilt,
}

pub fn run(options: Options, db_url: &str) -> anyhow::Result<Branch> {
  // Migrations don't support async connection, and this function doesn't need to be async
  let mut conn = PgConnection::establish(db_url)?;

  // If possible, skip getting a lock and recreating the "r" schema, so
  // lemmy_server processes in a horizontally scaled setup can start without causing locks
  if !options.revert
    && options.run
    && options.limit.is_none()
    && !conn
      .has_pending_migration(migrations())
      .map_err(convert_err)?
  {
    // The condition above implies that the migration that creates the previously_run_sql table was
    // already run
    let sql_unchanged = exists(
      previously_run_sql::table.filter(previously_run_sql::content.eq(replaceable_schema())),
    );

    let schema_exists = exists(pg_namespace::table.find("r"));

    if select(sql_unchanged.and(schema_exists)).get_result(&mut conn)? {
      return Ok(Branch::EarlyReturn);
    }
  }

  // Block concurrent attempts to run migrations until `conn` is closed and disable the
  // trigger that prevents the Diesel CLI from running migrations
  options.print("Waiting for lock...");
  conn.batch_execute("SELECT pg_advisory_lock(0);")?;
  options.print("Running Database migrations (This may take a long time)...");

  // Drop `r` schema, so migrations don't need to be made to work both with and without things in
  // it existing
  revert_replaceable_schema(&mut conn)?;

  run_selected_migrations(&mut conn, &options).map_err(convert_err)?;

  // Only run replaceable_schema if the newest migration was applied
  let output = if (options.run && options.limit.is_none())
    || !conn
      .has_pending_migration(migrations())
      .map_err(convert_err)?
  {
    #[cfg(test)]
    if options.enable_diff_check {
      let before = diff_check::get_dump();

      run_replaceable_schema(&mut conn)?;
      revert_replaceable_schema(&mut conn)?;

      let after = diff_check::get_dump();

      diff_check::check_dump_diff([&before, &after], "The code in crates/db_schema_setup/replaceable_schema incorrectly created or modified things outside of the `r` schema, causing these changes to be left behind after dropping the schema:");

      diff_check::deferr_constraint_check(&after);
    }

    run_replaceable_schema(&mut conn)?;

    Branch::ReplaceableSchemaRebuilt
  } else {
    Branch::ReplaceableSchemaNotRebuilt
  };

  options.print("Database migrations complete.");

  Ok(output)
}

fn run_replaceable_schema(conn: &mut PgConnection) -> anyhow::Result<()> {
  conn.transaction(|conn| {
    conn
      .batch_execute(&replaceable_schema())
      .with_context(|| format!("Failed to run SQL files in {REPLACEABLE_SCHEMA_PATH}"))?;

    let num_rows_updated = update(previously_run_sql::table)
      .set(previously_run_sql::content.eq(replaceable_schema()))
      .execute(conn)?;

    debug_assert_eq!(num_rows_updated, 1);

    Ok(())
  })
}

fn revert_replaceable_schema(conn: &mut PgConnection) -> anyhow::Result<()> {
  conn
    .batch_execute("DROP SCHEMA IF EXISTS r CASCADE;")
    .with_context(|| format!("Failed to revert SQL files in {REPLACEABLE_SCHEMA_PATH}"))?;

  // Value in the ` previously_run_sql ` table is not set here because the table might not exist,
  // and that's fine because the existence of the `r` schema is also checked

  Ok(())
}

fn run_selected_migrations(
  conn: &mut PgConnection,
  options: &Options,
) -> diesel::migration::Result<()> {
  let mut wrapper = MigrationHarnessWrapper {
    conn,
    options,
    #[cfg(test)]
    enable_diff_check: options.enable_diff_check,
  };

  if options.revert {
    if let Some(limit) = options.limit {
      for _ in 0..limit {
        wrapper.revert_last_migration(migrations())?;
      }
    } else {
      wrapper.revert_all_migrations(migrations())?;
    }
  }

  if options.run {
    if let Some(limit) = options.limit {
      for _ in 0..limit {
        wrapper.run_next_migration(migrations())?;
      }
    } else {
      wrapper.run_pending_migrations(migrations())?;
    }
  }

  Ok(())
}

/// Makes `diesel::migration::Result` work with `anyhow` and `FastJobError`
fn convert_err(e: Box<dyn std::error::Error + Send + Sync>) -> anyhow::Error {
  anyhow!(e)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
  use super::{
    Branch::{EarlyReturn, ReplaceableSchemaNotRebuilt, ReplaceableSchemaRebuilt},
    *,
  };
  use lemmy_utils::{error::FastJobResult, settings::SETTINGS};
  use serial_test::serial;
  // The number of migrations that should be run to set up some test data.
  // Currently, this includes migrations until
  // 2020-04-07-135912_add_user_category_apub_constraints, since there are some mandatory apub
  // fields need to be added.

  const INITIAL_MIGRATIONS_COUNT: u64 = 40;

  // Test data IDs
  const TEST_USER_ID_1: i32 = 101;
  const USER1_NAME: &str = "test_user_1";
  const USER1_ACTOR_ID: &str = "test_user_1@fedi.example";
  const USER1_PREFERRED_NAME: &str = "preferred_1";
  const USER1_EMAIL: &str = "email1@example.com";
  const USER1_PASSWORD: &str = "test_password_1";
  const USER1_PUBLIC_KEY: &str = "test_public_key_1";

  const TEST_USER_ID_2: i32 = 102;
  const USER2_NAME: &str = "test_user_2";
  const USER2_ACTOR_ID: &str = "test_user_2@fedi.example";
  const USER2_PREFERRED_NAME: &str = "preferred2";
  const USER2_EMAIL: &str = "email2@example.com";
  const USER2_PASSWORD: &str = "test_password_2";
  const USER2_PUBLIC_KEY: &str = "test_public_key_2";

  const TEST_category_ID_1: i32 = 101;
  const category_NAME: &str = "test_category_1";
  const category_TITLE: &str = "Test Category 1";
  const category_DESCRIPTION: &str = "This is a test category.";
  const CATEGORY_ID: i32 = 4; // Should be a valid category "Movies"
  const category_ACTOR_ID: &str = "https://fedi.example/category/12345";
  const category_PUBLIC_KEY: &str = "test_public_key_category_1";

  const TEST_POST_ID_1: i32 = 101;
  const POST_NAME: &str = "Post Title";
  const POST_URL: &str = "https://fedi.example/post/12345";
  const POST_BODY: &str = "Post Body.";

  const TEST_COMMENT_ID_1: i32 = 101;
  const COMMENT1_CONTENT: &str = "Comment";

  const TEST_COMMENT_ID_2: i32 = 102;
  const COMMENT2_CONTENT: &str = "Reply";

  #[test]
  #[serial]
  fn test_schema_setup() -> FastJobResult<()> {
    let o = Options::default();
    let db_url = SETTINGS.get_database_url();
    let mut conn = PgConnection::establish(&db_url)?;

    // Start with a consistent state by dropping everything
    conn.batch_execute("DROP OWNED BY CURRENT_USER;")?;

    // Run initial migrations to prepare basic tables
    assert_eq!(
      run(o.run().limit(INITIAL_MIGRATIONS_COUNT), &db_url)?,
      ReplaceableSchemaNotRebuilt
    );

    // Insert the test data
    insert_test_data(&mut conn)?;

    // Run all migrations and make sure that changes can be correctly reverted
    assert_eq!(
      run(o.run().enable_diff_check(), &db_url)?,
      ReplaceableSchemaRebuilt
    );

    // Check for early return
    assert_eq!(run(o.run(), &db_url)?, EarlyReturn);

    // Test `limit`
    assert_eq!(
      run(o.revert().limit(1), &db_url)?,
      ReplaceableSchemaNotRebuilt
    );
    assert_eq!(
      conn
        .pending_migrations(migrations())
        .map_err(convert_err)?
        .len(),
      1
    );
    assert_eq!(run(o.run().limit(1), &db_url)?, ReplaceableSchemaRebuilt);

    // This should throw an error saying to use lemmy_server instead of diesel CLI
    conn.batch_execute("DROP OWNED BY CURRENT_USER;")?;
    assert!(matches!(
      conn.run_pending_migrations(migrations()),
      Err(e) if e.to_string().contains("app_108jobs_api_server")
    ));

    // Diesel CLI's way of running migrations shouldn't break the custom migration runner
    assert_eq!(run(o.run(), &db_url)?, ReplaceableSchemaRebuilt);

    Ok(())
  }

  fn insert_test_data(conn: &mut PgConnection) -> FastJobResult<()> {
    // Users
    conn.batch_execute(&format!(
      "INSERT INTO user_ (id, name, actor_id, preferred_username, password_encrypted, email, public_key) \
          VALUES ({}, '{}', '{}', '{}', '{}', '{}', '{}')",
      TEST_USER_ID_1,
      USER1_NAME,
      USER1_ACTOR_ID,
      USER1_PREFERRED_NAME,
      USER1_PASSWORD,
      USER1_EMAIL,
      USER1_PUBLIC_KEY
    ))?;

    conn.batch_execute(&format!(
      "INSERT INTO user_ (id, name, actor_id, preferred_username, password_encrypted, email, public_key) \
          VALUES ({}, '{}', '{}', '{}', '{}', '{}', '{}')",
      TEST_USER_ID_2,
      USER2_NAME,
      USER2_ACTOR_ID,
      USER2_PREFERRED_NAME,
      USER2_PASSWORD,
      USER2_EMAIL,
      USER2_PUBLIC_KEY
    ))?;

    // Category
    conn.batch_execute(&format!(
      "INSERT INTO category (id, actor_id, public_key, name, title, description, category_id, creator_id) \
          VALUES ({}, '{}', '{}', '{}', '{}', '{}', {}, {})",
      TEST_category_ID_1,
      category_ACTOR_ID,
      category_PUBLIC_KEY,
      category_NAME,
      category_TITLE,
      category_DESCRIPTION,
      CATEGORY_ID,
      TEST_USER_ID_1
    ))?;

    conn.batch_execute(&format!(
      "INSERT INTO category_moderator (category_id, user_id) \
          VALUES ({}, {})",
      TEST_category_ID_1, TEST_USER_ID_1
    ))?;

    // Post
    conn.batch_execute(&format!(
      "INSERT INTO post (id, name, url, body, creator_id, category_id) \
          VALUES ({}, '{}', '{}', '{}', {}, {})",
      TEST_POST_ID_1,
      POST_NAME,
      POST_URL,
      POST_BODY,
      TEST_USER_ID_1,
      TEST_category_ID_1,
    ))?;

    // Comment
    conn.batch_execute(&format!(
      "INSERT INTO comment (id, creator_id, post_id, parent_id, content) \
           VALUES ({}, {}, {}, NULL, '{}')",
      TEST_COMMENT_ID_1, TEST_USER_ID_2, TEST_POST_ID_1, COMMENT1_CONTENT
    ))?;

    conn.batch_execute(&format!(
      "INSERT INTO comment (id, creator_id, post_id, parent_id, content) \
           VALUES ({}, {}, {}, {}, '{}')",
      TEST_COMMENT_ID_2,
      TEST_USER_ID_1,
      TEST_POST_ID_1,
      TEST_COMMENT_ID_1,
      COMMENT2_CONTENT,
    ))?;

    conn.batch_execute(&format!(
      "INSERT INTO comment_like (user_id, comment_id, post_id, score) \
           VALUES ({}, {}, {}, {})",
      TEST_USER_ID_1, TEST_COMMENT_ID_1, TEST_POST_ID_1, 1
    ))?;

    Ok(())
  }

}
