use syn::ItemImpl;

pub(crate) struct SystemImpl {
    crate_name: String,
    item: ItemImpl,
}
