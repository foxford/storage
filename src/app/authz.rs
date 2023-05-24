use svc_authz::IntentObject;

#[derive(Clone)]
pub struct AuthzObject {
    object: Vec<String>,
}

impl AuthzObject {
    pub fn new(obj: &[&str]) -> Self {
        Self {
            object: obj.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn from_owned_slice(obj: &[String]) -> Self {
        Self {
            object: Vec::from(obj),
        }
    }
}

impl IntentObject for AuthzObject {
    fn to_ban_key(&self) -> Option<Vec<String>> {
        None
    }

    fn to_vec(&self) -> Vec<String> {
        self.object.clone()
    }

    fn box_clone(&self) -> Box<dyn IntentObject> {
        Box::new(self.clone())
    }
}

impl From<AuthzObject> for Box<dyn IntentObject> {
    fn from(o: AuthzObject) -> Self {
        Box::new(o)
    }
}
