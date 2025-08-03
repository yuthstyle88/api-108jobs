use either::Either;
use crate::objects::comment::ApubComment;
use crate::objects::community::ApubCommunity;
use crate::objects::instance::ApubSite;
use crate::objects::person::ApubPerson;
use crate::objects::post::ApubPost;

pub mod comment;
pub mod community;
pub mod instance;
pub mod person;
pub mod post;

pub type SearchableObjects = Either<PostOrComment, UserOrCommunity>;

pub type PostOrComment = Either<ApubPost, ApubComment>;

pub type UserOrCommunity = Either<ApubPerson, ApubCommunity>;

pub type SiteOrMultiOrCommunityOrUser = Either<ApubSite, UserOrCommunity>;