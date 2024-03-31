use std::collections::HashMap;
use crate::compiler::types::TypeBase;

#[derive(Clone)]
struct Container {
    pub trait_object: Box<dyn TypeBase>,
}
pub struct VariableCache {
    map: HashMap<String, Container>,
    local: HashMap<i32, Vec<String>>,
}

impl Default for VariableCache {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableCache {
    pub fn new() -> Self {
        VariableCache {
            map: HashMap::new(),
            local: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, trait_object: Box<dyn TypeBase>, depth: i32) {
        let mut locals: HashMap<i32, bool> = HashMap::new();
        locals.insert(depth, true);
        self.map.insert(key.to_string(), Container { trait_object });
        match self.local.get(&depth) {
            Some(val) => {
                let mut val_clone = val.clone();
                val_clone.push(key.to_string());
                self.local.insert(depth, val_clone);
            }
            None => {
                self.local.insert(depth, vec![key.to_string()]);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<Box<dyn TypeBase>> {
        match self.map.get(key) {
            Some(v) => Some(dyn_clone::clone_box(&*v.trait_object)),
            None => None,
        }
    }

    #[allow(dead_code)]
    fn del(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn del_locals(&mut self, depth: i32) {
        if let Some(v) = self.local.get(&depth) {
            for local in v.iter() {
                self.map.remove(&local.to_string());
            }
            self.local.remove(&depth);
        }
    }
}