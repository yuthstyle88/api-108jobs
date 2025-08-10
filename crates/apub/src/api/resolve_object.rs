use crate::fetcher::search::{search_query_to_object_id, search_query_to_object_id_local};
use actix_web::web::{Data, Json, Query};
use either::Either::*;
use lemmy_api_utils::{ utils::check_private_instance};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_search_combined::{SearchCombinedView, SearchResponse};
use lemmy_db_views_site::{api::ResolveObject, SiteView};
use lemmy_utils::error::{FastJobErrorExt2, FastJobErrorType, FastJobResult};

pub async fn resolve_object(
    data: Query<ResolveObject>,
    context: Data<FastJobContext>,
    local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<SearchResponse>> {
    let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
    check_private_instance(&local_user_view, &local_site)?;

    let res = resolve_object_internal(&data.q, &local_user_view, &context).await?;
    Ok(Json(SearchResponse {
        results: vec![res],
        ..Default::default()
    }))
}

pub(super) async fn resolve_object_internal(
    query: &str,
    local_user_view: &Option<LocalUserView>,
    context: &Data<FastJobContext>,
) -> FastJobResult<SearchCombinedView> {
    use SearchCombinedView::*;

    // If we get a valid personId back we can safely assume that the user is authenticated,
    // if there's no personId then the JWT was missing or invalid.
    let is_authenticated = local_user_view.is_some();

    let object = if is_authenticated || cfg!(debug_assertions) {
        // user is fully authenticated; allow remote lookups as well.
        search_query_to_object_id(query.to_string(), context).await
    } else {
        // user isn't authenticated only allow a local search.
        search_query_to_object_id_local(query, context).await
    }
        .with_fastjob_type(FastJobErrorType::NotFound)?;

    let my_person_id = local_user_view.as_ref().map(|l| l.person.id);
    let local_user = local_user_view.as_ref().map(|l| l.local_user.clone());
    let is_admin = local_user.as_ref().map(|l| l.admin).unwrap_or_default();
    let pool = &mut context.pool();
    let local_instance_id = SiteView::read_local(pool).await?.site.instance_id;

    Ok(match object {
        Left(Left(Left(p))) => {
            Post(PostView::read(pool, p.id, local_user.as_ref(), local_instance_id).await?)
        }
        Left(Left(Right(c))) => {
            Comment(CommentView::read(pool, c.id, local_user.as_ref(), local_instance_id).await?)
        }
        Left(Right(Left(u))) => {
            Person(PersonView::read(pool, u.id, my_person_id, local_instance_id, is_admin).await?)
        }
        Left(Right(Right(c))) => {
            Community(CommunityView::read(pool, c.id, local_user.as_ref()).await?)
        }
    })
}