use nix::unistd;

pub fn check_root() -> Result<(), &'static str> {
    if unistd::geteuid().is_root() {
        Ok(())
    } else {
        Err("Not running as root (LPE exploits are a premium feature)")
    }
}
