use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{bail, Result};

use crate::{
    class::Class,
    lli::{
        app_loader::AppLoader, class_loader::BootstrapLoader, loader::ClassLoader,
        response::Response,
    },
    method_area::MethodArea,
};

//FIXME I don't like the Response API. Response::NotFound functions as an Err,
// and Response::NotFound gets mapped to an Err. It all feels very redundant.
// I should either be returning an Err or a Response.

const DEFAULT_CAPACITY: usize = 64;

pub(crate) struct ClassManager {
    loaders: Vec<Box<dyn ClassLoader>>,

    by_name: HashMap<String, usize>,
    by_id: HashMap<usize, usize>,
    count: AtomicUsize,
}

impl ClassManager {
    pub(crate) fn new(jdk: PathBuf, c_path: PathBuf) -> Self {
        Self {
            by_name: HashMap::with_capacity(DEFAULT_CAPACITY),
            by_id: HashMap::with_capacity(DEFAULT_CAPACITY),
            count: AtomicUsize::new(0),
            loaders: vec![
                Box::new(BootstrapLoader::new(jdk)),
                Box::new(AppLoader::new(c_path)),
            ],
        }
    }

    pub(crate) fn request(&mut self, name: &str, met_area: &mut MethodArea) -> Result<Response> {
        let response = self.resolve_and_index(name, met_area)?;
        if let Response::NonFound = response {
            bail!("Class not found: {name}.");
        } else {
            Ok(response)
        }
    }

    #[inline]
    pub(crate) fn resolve_and_index(
        &mut self,
        name: &str,
        met_area: &mut MethodArea,
    ) -> Result<Response> {
        if !self.by_name.contains_key(name) {
            for loader in self.loaders.iter_mut() {
                let c_file = match loader.get(name) {
                    Ok(class_file) => class_file,
                    Err(_) => {
                        continue;
                    }
                };
                let class = Class::from(&c_file);
                match self.store_class(class, met_area) {
                    // since the class had to retrieved and stored it has not been initialized.
                    Ok((class, index)) => return Ok(Response::InitReq(class, index)),
                    Err(_) => {
                        bail!("MethodArea allocation error.")
                    }
                }
            }

            bail!("Class: {name} not found!");
        } else {
            Ok(Response::Ready(*self.by_name.get(name).unwrap()))
        }
    }

    #[inline]
    fn store_class(
        &mut self,
        class: Class,
        met_area: &mut MethodArea,
    ) -> Result<(&'static Class, usize)> {
        let name = class.get_name();
        let index = met_area.push(class)?;
        let id = self.count.load(Ordering::SeqCst);
        self.count.store(id + 1, Ordering::SeqCst);
        self.by_id.insert(id, index);
        self.by_name.insert(name, index);
        Ok((met_area.get_class(index)?, index))
    }
}
