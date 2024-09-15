use std::{borrow::BorrowMut, collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct BindUnique {
    bind_map: HashMap<PathBuf, PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Bind {
    pub src: PathBuf,
    pub dist: PathBuf,
}

impl BindUnique {
    pub fn new(binds: &Vec<Bind>) -> BindUnique {
        let mut ret = BindUnique {
            bind_map: HashMap::new(),
        };
        binds.iter().for_each(|Bind { src, dist }| {
            ret.bind_map
                .borrow_mut()
                .insert(dist.clone(), src.clone())
                .inspect(|old_src| {
                    panic!("Multiple bind {dist:?} from {old_src:?} to {src:?} ");
                });
        });
        ret
    }
}
