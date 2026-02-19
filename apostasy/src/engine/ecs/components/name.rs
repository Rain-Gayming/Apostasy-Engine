use apostasy_macros::Component;

use crate as apostasy;

#[derive(Component)]
pub struct Name(pub String);

impl Default for Name {
    fn default() -> Self {
        Self("Entity".to_string())
    }
}
