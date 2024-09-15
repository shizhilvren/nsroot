use std::{env, fs, os::unix::process::CommandExt, path::PathBuf, process};

use log::{debug, error, warn};
use nix::{
    mount::{mount, MsFlags},
    sched::{unshare, CloneFlags},
    unistd,
};

use crate::bind::{self, Bind};

const NONE: Option<&'static [u8]> = None;

#[derive(Debug)]
pub struct Chroot {
    pub rootdir: PathBuf,
    pub bind_set: bind::BindUnique,
}

impl Chroot {
    pub fn run_chroot(self: &Self, cmd: &str, args: &[String]) {
        let cwd = env::current_dir().expect("cannot get current working directory");
        let cwd = PathBuf::from("/");

        let uid = unistd::getuid();
        let gid = unistd::getgid();

        unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUSER).expect("unshare failed");
        self.bind_all();
        // chroot
        unistd::chroot(self.rootdir.as_path())
            .unwrap_or_else(|err| panic!("chroot({}): {}", self.rootdir.display(), err));

        env::set_current_dir("/").expect("cannot change directory to /");

        // // fixes issue #1 where writing to /proc/self/gid_map fails
        // // see user_namespaces(7) for more documentation
        // if let Ok(mut file) = fs::File::create("/proc/self/setgroups") {
        //     let _ = file.write_all(b"deny");
        // }

        // let mut uid_map =
        //     fs::File::create("/proc/self/uid_map").expect("failed to open /proc/self/uid_map");
        // uid_map
        //     .write_all(format!("{} {} 1", uid, uid).as_bytes())
        //     .expect("failed to write new uid mapping to /proc/self/uid_map");

        // let mut gid_map =
        //     fs::File::create("/proc/self/gid_map").expect("failed to open /proc/self/gid_map");
        // gid_map
        //     .write_all(format!("{} {} 1", gid, gid).as_bytes())
        //     .expect("failed to write new gid mapping to /proc/self/gid_map");

        // let args: [String] = [];
        // restore cwd
        env::set_current_dir(&cwd)
            .unwrap_or_else(|_| panic!("cannot restore working directory {}", cwd.display()));

        let err = process::Command::new(cmd)
            .args(args)
            .env("NIX_CONF_DIR", "/nix/etc/nix")
            .exec();

        eprintln!("failed to execute {}: {}", &cmd, err);
        process::exit(1);
    }

    pub fn bind_all(self: &Self) {
        self.bind_set.bind_map.iter().for_each(|(dist, src)| {
            self.bind(&Bind {
                src: src.clone(),
                dist: dist.clone(),
            });
        })
    }
    pub fn bind(self: &Self, bind: &Bind) {
        let Bind { src, dist } = bind;
        let bind_str = format!("bind host {} to guest {}", src.display(), dist.display());
        debug!("{bind_str}");
        if !bind.is_root_path() {
            panic!("{bind_str}, but some path not have root");
        }
        let real_dist = self.rootdir.join(
            dist.strip_prefix("/")
                .expect(format!("dist {} not have root", dist.display()).as_str()),
        );
        debug!("real_dist : {real_dist:?}");
        if !dist.as_path().exists() {
            fs::create_dir_all(&real_dist).unwrap_or_else(|err| {
                panic!("failed to create guest {} : {}", &dist.display(), err)
            });
        }
        let real_dist_is_empty_dir = real_dist
            .read_dir()
            .and_then(|ref mut v| Ok(v.next().is_none()))
            .map_or_else(|_| false, |v| v);
        if !real_dist_is_empty_dir {
            warn!("{bind_str}, but {} is not empty dir", dist.display());
        }
        mount(
            Some(src),
            &real_dist,
            Some("none"),
            MsFlags::MS_BIND | MsFlags::MS_REC,
            NONE,
        )
        .unwrap_or_else(|err| panic!("failed to {bind_str} : {}", err));
    }
}
