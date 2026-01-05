use crate::objects::person::ApubPerson;
use crate::objects::post::ApubPost;
use either::Either;

pub mod comment;
pub mod category;
pub mod instance;
pub mod person;
pub mod post;

pub type SearchableObjects = Either<ApubPost, ApubPerson>;