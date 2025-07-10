use std::collections::HashMap;
use serde::Deserialize;
use crate::loader::Lang;

#[derive(Debug, Deserialize)]
pub struct NamespaceTranslations(pub HashMap<String, String>);

pub type LangTranslations = HashMap<String, NamespaceTranslations>;
pub type AllTranslations = HashMap<Lang, LangTranslations>;
