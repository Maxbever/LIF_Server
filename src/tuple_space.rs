use std::sync::{Arc, Mutex};

use rustupolis::space::Space;
use rustupolis::store::SimpleStore;

pub struct TupleSpace {
    tuple_space: Arc<Mutex<Space<SimpleStore>>>,
    tuple_space_name: String,
    attributes: Vec<String>,
}

impl TupleSpace {
    pub fn new(
        tuple_space: Arc<Mutex<Space<SimpleStore>>>,
        attributes: Vec<String>,
        tuple_space_name: &str,
    ) -> TupleSpace {
        let tuple_space_name = String::from(tuple_space_name);
        TupleSpace {
            tuple_space,
            tuple_space_name,
            attributes,
        }
    }

    pub fn tuple_space(&self) -> &Arc<Mutex<Space<SimpleStore>>> {
        &self.tuple_space
    }

    pub fn attributes(&self) -> &Vec<String> {
        &self.attributes
    }

    pub fn tuple_space_name(&self) -> &str {
        &self.tuple_space_name
    }
}
