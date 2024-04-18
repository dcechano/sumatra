use anyhow::Result;

use crate::{
    lli::{class_loader::ClassLoader, response::Response},
    method_area::MethodArea,
};

pub(crate) struct ClassManager {
    class_loader: ClassLoader,
}

impl ClassManager {
    pub(crate) fn new() -> Self {
        Self {
            class_loader: ClassLoader::new(),
        }
    }

    pub(crate) fn request(&mut self, name: &str, met_area: &mut MethodArea) -> Result<Response> {
        let response = self.class_loader.resolve_and_index(name, met_area)?;
        if let Response::NonFound = response {
            todo!(
                "Eventually when the ClassManager manages several ClassLoaders, this would mean \
                trying a different ClassLoader."
            );
        } else {
            Ok(response)
        }
    }
}
