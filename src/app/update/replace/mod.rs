pub mod bundle;
pub mod check;
pub mod signed;
pub mod staged;
pub mod swap;

use std::io;
use std::path::Path;

use super::download::Progress;
use super::route::Route;
use super::version::Version;

pub fn put_in_place(
    route: Route,
    exe: &Path,
    version: Version,
    shown: Progress,
    before_swap: &dyn Fn(),
) -> io::Result<()> {
    let target = route.target(exe);
    staged::fetch(route, &target, version, shown)?;
    before_swap();
    swap::replace(&target, &staged::path(&target, version))
}
