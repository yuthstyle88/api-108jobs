use crate::{
  protocol::note::Note,
  utils::{
    functions::{
      append_attachments_to_comment,
      check_apub_id_valid_with_strictness,
      generate_to,
      read_from_string_or_source,
      verify_person_in_community,
    },
    markdown_links::markdown_rewrite_remote_links,
    protocol::{InCommunity, LanguageTag, Source},
  },
};
use activitypub_federation::{
  config::Data,
  kinds::object::NoteType,
  protocol::{
    values::MediaTypeMarkdownOrHtml,
    verification::{verify_domains_match, verify_is_remote_object},
  },
  traits::Object,
};
use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::FastJobContext,
  plugins::{plugin_hook_after},
  utils::{
    check_comment_depth,
    check_is_mod_or_admin,
    get_url_blocklist,
    process_markdown,
    slur_regex,
  },
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentInsertForm, CommentUpdateForm},
    community::Community,
    person::Person,
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FastJobError, FastJobResult},
  utils::markdown::markdown_to_html,
};
use std::ops::Deref;
use url::Url;
use lemmy_utils::error::FastJobErrorType;

#[derive(Clone, Debug)]
pub struct ApubComment(pub Comment);

impl Deref for ApubComment {
  type Target = Comment;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Comment> for ApubComment {
  fn from(c: Comment) -> Self {
    ApubComment(c)
  }
}

#[async_trait::async_trait]
impl Object for ApubComment {
  type DataType = FastJobContext;
  type Kind = Note;
  type Error = FastJobError;

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> FastJobResult<Option<Self>> {
    Ok(
      Comment::read_from_apub_id(&mut context.pool(), object_id)
       .await?
       .map(Into::into),
    )
  }

  async fn delete(self, context: &Data<Self::DataType>) -> FastJobResult<()> {
    if !self.deleted {
      let form = CommentUpdateForm {
        deleted: Some(true),
        ..Default::default()
      };
      Comment::update(&mut context.pool(), self.id, &form).await?;
    }
    Ok(())
  }

  fn is_deleted(&self) -> bool {
    self.removed || self.deleted
  }

  async fn into_json(self, context: &Data<Self::DataType>) -> FastJobResult<Note> {
    let creator_id = self.creator_id;
    let creator = Person::read(&mut context.pool(), creator_id).await?;

    let post_id = self.post_id;
    let post = Post::read(&mut context.pool(), post_id).await?;
    let community_id = post.community_id;
    let community = Community::read(&mut context.pool(), community_id).await?;

    let in_reply_to = if let Some(comment_id) = self.parent_comment_id() {
      let parent_comment = Comment::read(&mut context.pool(), comment_id).await?;
      parent_comment.ap_id.inner().clone().into()
    } else {
      post.ap_id.inner().clone().into()
    };
    let language = Some(LanguageTag::new_single(self.language_id, &mut context.pool()).await?);

    let note = Note {
      r#type: NoteType::Note,
      id: self.ap_id.inner().clone().into(),
      attributed_to: creator.ap_id.inner().clone().into(),
      to: generate_to(&community)?,
      content: markdown_to_html(&self.content),
      media_type: Some(MediaTypeMarkdownOrHtml::Html),
      source: Some(Source::new(self.content.clone())),
      in_reply_to,
      published: Some(self.published_at),
      updated: self.updated_at,
      distinguished: Some(self.distinguished),
      language,
      attachment: vec![],
    };

    Ok(note)
  }

  /// Recursively fetches all parent comments. This can lead to a stack overflow so we need to
  /// Box::pin all large futures on the heap.
  async fn verify(
    note: &Note,
    expected_domain: &Url,
    context: &Data<FastJobContext>,
  ) -> FastJobResult<()> {
    verify_domains_match(note.id.inner(), expected_domain)?;
    verify_domains_match(note.attributed_to.inner(), note.id.inner())?;
    let community = Box::pin(note.community(context)).await?;

    Box::pin(check_apub_id_valid_with_strictness(
      note.id.inner(),
      community.local,
      context,
    ))
     .await?;
    if let Err(e) = verify_is_remote_object(&note.id, context) {
      if let Ok(comment) = note.id.dereference_local(context).await {
        comment.set_not_pending(&mut context.pool()).await?;
      }
      return Err(e.into());
    }
    Box::pin(verify_person_in_community(
      &note.attributed_to,
      &community,
      context,
    ))
     .await?;

    let (post, _) = Box::pin(note.get_parents(context)).await?;
    let creator = Box::pin(note.attributed_to.dereference(context)).await?;
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    let local_instance_id = site_view.site.instance_id;

    let is_mod_or_admin = check_is_mod_or_admin(
      &mut context.pool(),
      creator.id,
      local_instance_id,
    )
     .await
     .is_ok();
    if post.locked && !is_mod_or_admin {
      Err(FastJobErrorType::PostIsLocked)?
    } else {
      Ok(())
    }
  }

  /// Converts a `Note` to `Comment`.
  ///
  /// If the parent community, post and comment(s) are not known locally, these are also fetched.
  async fn from_json(note: Note, context: &Data<FastJobContext>) -> FastJobResult<ApubComment> {
    let creator = note.attributed_to.dereference(context).await?;
    let (post, parent_comment) = note.get_parents(context).await?;
    if let Some(c) = &parent_comment {
      check_comment_depth(c)?;
    }

    let content = read_from_string_or_source(&note.content, &note.media_type, &note.source);

    let slur_regex = slur_regex(context).await?;
    let url_blocklist = get_url_blocklist(context).await?;
    let content = append_attachments_to_comment(content, &note.attachment, context).await?;
    let content = process_markdown(&content, &slur_regex, &url_blocklist, context).await?;
    let content = markdown_rewrite_remote_links(content, context).await;
    let language_id = Some(
      LanguageTag::to_language_id_single(note.language.unwrap_or_default(), &mut context.pool())
       .await?,
    );

    let form = CommentInsertForm {
      creator_id: creator.id,
      post_id: post.id,
      content,
      removed: None,
      published_at: note.published,
      updated_at: note.updated,
      deleted: Some(false),
      ap_id: Some(note.id.inner().clone().into()),
      distinguished: note.distinguished,
      local: Some(false),
      language_id,
      pending: Some(false),
    };
    let parent_comment_path = parent_comment.map(|t| t.0.path);
    let timestamp: DateTime<Utc> = note.updated.or(note.published).unwrap_or_else(Utc::now);
    let comment = Comment::insert_apub(
      &mut context.pool(),
      Some(timestamp),
      &form,
      parent_comment_path.as_ref(),
    )
     .await?;
    plugin_hook_after("after_receive_federated_comment", &comment)?;
    Ok(comment.into())
  }
}

