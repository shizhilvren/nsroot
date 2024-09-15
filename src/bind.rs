use std::{borrow::BorrowMut, collections::HashMap, env, path::PathBuf};

#[derive(Debug)]
pub struct BindUnique {
    pub bind_map: HashMap<PathBuf, PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Bind {
    pub src: PathBuf,
    pub dist: PathBuf,
}

impl BindUnique {
    pub fn new(binds: &Vec<Bind>) -> BindUnique {
        let cwd = env::current_dir().expect("cannot get current working directory");
        let mut ret = BindUnique {
            bind_map: HashMap::new(),
        };
        binds.iter().for_each(|bind| {
            let Bind { src, dist } = bind.map_to_abs(&cwd);
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

impl Bind {
    pub fn map_to_abs(self: &Self, host_cwd: &PathBuf) -> Bind {
        let src = if self.src.has_root() {
            self.src.clone()
        } else {
            host_cwd.join(&self.src)
        };
        if !self.dist.has_root() {
            panic!(
                "bind src:{} to dist:{}, but dist is not a absolute path",
                src.display(),
                self.dist.display()
            );
        };
        Bind {
            src: src,
            dist: self.dist.clone(),
        }
    }
    pub fn is_root_path(self: &Self) -> bool {
        self.src.has_root() && self.dist.has_root()
    }
}
