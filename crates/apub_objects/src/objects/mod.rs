use crate::objects::{person::ApubPerson, post::ApubPost};
use either::Either;

pub mod category;
pub mod comment;
pub mod instance;
pub mod person;
pub mod post;

pub type SearchableObjects = Either<ApubPost, ApubPerson>;
