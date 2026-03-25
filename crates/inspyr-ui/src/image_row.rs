/* image_row.rs
 *
 * Copyright 2026 Gotam Gorabh
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use glib_macros::Properties;
use gtk::glib;
use gtk::glib::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use inspyr_database::Image;

mod imp {
    use super::*;
    use glib::prelude::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::InspyrImageRow)]
    pub struct InspyrImageRow {
        #[property(get, set)]
        pub id: Cell<i64>,
        #[property(get, set)]
        pub path: RefCell<String>,
        #[property(get, set)]
        pub filename: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InspyrImageRow {
        const NAME: &'static str = "InspyrImageRow";
        type Type = super::InspyrImageRow;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for InspyrImageRow {}
}

glib::wrapper! {
    pub struct InspyrImageRow(ObjectSubclass<imp::InspyrImageRow>);
}

impl InspyrImageRow {
    pub fn from_image(image: &Image) -> Self {
        glib::Object::builder()
            .property("id", image.id)
            .property("path", image.path.to_string_lossy().as_ref())
            .property("filename", image.filename.as_str())
            .build()
    }
}
