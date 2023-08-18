#![allow(clippy::option_if_let_else)]

use darling::util::PathList;
use darling::FromMeta;

#[derive(FromMeta)]
pub(super) struct TestArgs {
    #[darling(default)]
    pub(super) disabled: PathList,
}
