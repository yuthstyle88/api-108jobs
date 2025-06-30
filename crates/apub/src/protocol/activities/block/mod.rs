pub mod block_user;
pub mod undo_block_user;

#[cfg(test)]
mod tests {
  use crate::protocol::activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser};
  use lemmy_apub_objects::utils::test::test_parse_lemmy_item;
  use lemmy_utils::error::FastJobResult;

  #[test]
  fn test_parse_lemmy_block() -> FastJobResult<()> {
    test_parse_lemmy_item::<BlockUser>("assets/lemmy/activities/block/block_user.json")?;
    test_parse_lemmy_item::<UndoBlockUser>("assets/lemmy/activities/block/undo_block_user.json")?;
    Ok(())
  }
}
