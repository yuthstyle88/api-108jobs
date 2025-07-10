use std::collections::HashMap;
use serde::Deserialize;
use crate::loader::Lang;

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceTranslations(pub HashMap<String, String>);

pub type LangTranslations = HashMap<String, NamespaceTranslations>;
pub type AllTranslations = HashMap<Lang, LangTranslations>;
