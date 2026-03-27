/* photo_item.rs
 *
 * Copyright 2026 Gotam Gorabh
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::gio;
use gtk::{glib, CompositeTemplate};
use std::cell::Cell;
use std::path::Path;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/org/gnome/Inspyr/photo-item.ui")]
    #[properties(wrapper_type = super::InspyrPhotoItem)]
    pub struct InspyrPhotoItem {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,

        #[property(get, set)]
        icon_size: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InspyrPhotoItem {
        const NAME: &'static str = "InspyrPhotoItem";
        type Type = super::InspyrPhotoItem;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for InspyrPhotoItem {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.connect_icon_size_notify(gtk::glib::clone!(
                #[weak]
                obj,
                move |_| {
                    let s = obj.icon_size();
                    if s > 0 {
                        obj.set_size_request(s as i32, s as i32);
                    }
                }
            ));
            let s = obj.icon_size();
            if s > 0 {
                obj.set_size_request(s as i32, s as i32);
            }
        }
    }
    impl WidgetImpl for InspyrPhotoItem {}
    impl BinImpl for InspyrPhotoItem {}
}

glib::wrapper! {
    pub struct InspyrPhotoItem(ObjectSubclass<imp::InspyrPhotoItem>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for InspyrPhotoItem {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

impl InspyrPhotoItem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the image at `path`, or clear the tile if missing / invalid.
    pub fn load_from_path(&self, path: &Path) {
        let imp = imp::InspyrPhotoItem::from_obj(self);
        let picture = imp.picture.get();
        if path.exists() {
            picture.set_file(Some(&gio::File::for_path(path)));
        } else {
            picture.set_file(None::<&gio::File>);
        }
    }

    pub fn clear_thumbnail(&self) {
        let imp = imp::InspyrPhotoItem::from_obj(self);
        imp.picture.get().set_file(None::<&gio::File>);
    }
}
