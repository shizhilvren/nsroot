use std::{env, os::unix::process::CommandExt, path::PathBuf, process};

use nix::{
    mount::{mount, MsFlags},
    sched::{unshare, CloneFlags},
    unistd,
};

use crate::bind;

const NONE: Option<&'static [u8]> = None;

#[derive(Debug)]
pub struct Chroot {
    pub rootdir: PathBuf,
    pub bind_set: bind::BindUnique,
}

impl Chroot {
    pub fn run_chroot(self: Self, cmd: &str, args: &[String]) {
        let cwd = env::current_dir().expect("cannot get current working directory");
        let cwd = PathBuf::from("/");

        let uid = unistd::getuid();
        let gid = unistd::getgid();

        unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUSER).expect("unshare failed");

        let nixdir = PathBuf::from("/home/shizhilvren/nsroot");
        // mount the store
        let nix_mount = self.rootdir.join("nix");
        // fs::create_dir(&nix_mount)
        //     .unwrap_or_else(|err| panic!("failed to create {}: {}", &nix_mount.display(), err));
        mount(
            Some(&nixdir),
            &nix_mount,
            Some("none"),
            MsFlags::MS_BIND | MsFlags::MS_REC,
            NONE,
        )
        .unwrap_or_else(|err| panic!("failed to bind mount {} to /nix: {}", nixdir.display(), err));

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
}
