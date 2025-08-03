use either::Either;

pub mod comment;
pub mod community;
pub mod instance;
pub mod person;
pub mod post;

pub type SearchableObjects = Either<PostOrComment, UserOrCommunity>;

pub type PostOrComment = Either<ApubPost, ApubComment>;

pub type UserOrCommunity = Either<ApubPerson, ApubCommunity>;

pub type SiteOrMultiOrCommunityOrUser =
Either<ApubSite, UserOrCommunity>;