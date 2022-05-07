use crate::attributes;
use syn::{ImplItem, ItemImpl};

pub(crate) fn clean(block: &ItemImpl) -> ItemImpl {
    let mut cleaned_block = block.clone();
    cleaned_block.items = cleaned_block
        .items
        .into_iter()
        .map(clean_impl_item)
        .collect();
    cleaned_block
}

fn clean_impl_item(mut item: ImplItem) -> ImplItem {
    if let ImplItem::Method(method) = &mut item {
        method.attrs = method
            .attrs
            .clone()
            .into_iter()
            .filter(|a| attributes::parse_type(a).is_none())
            .collect();
    }
    item
}
