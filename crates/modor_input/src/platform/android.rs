use modor::ANDROID_APP;

pub(crate) fn open_virtual_keyboard() {
    if let Some(app) = ANDROID_APP.get() {
        app.show_soft_input(false)
    } else {
        log::error!("cannot open virtual keyboard (maybe modor::modor_main has not been used ?)");
    }
}

pub(crate) fn close_virtual_keyboard() {
    if let Some(app) = ANDROID_APP.get() {
        app.hide_soft_input(false)
    } else {
        log::error!("cannot open virtual keyboard (maybe modor::modor_main has not been used ?)");
    }
}
