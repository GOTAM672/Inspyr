/* photo_page.rs
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

use crate::image_row::InspyrImageRow;
use crate::photo_item::InspyrPhotoItem;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gio, glib, CompositeTemplate};
use inspyr_database::{Database, DatabaseOperations, ListOptions};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;

const LOG_DOMAIN: &str = "InspyrPhotoPage";

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/org/gnome/Inspyr/photo-page.ui")]
    #[properties(wrapper_type = super::InspyrPhotoPage)]
    pub struct InspyrPhotoPage {
        #[template_child]
        pub grid_view: TemplateChild<gtk::GridView>,

        #[property(get, set)]
        icon_size: Cell<u32>,

        /// Keeps the list model alive while the grid is shown.
        pub store: RefCell<Option<gio::ListStore>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InspyrPhotoPage {
        const NAME: &'static str = "InspyrPhotoPage";
        type Type = super::InspyrPhotoPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            InspyrImageRow::static_type();
            klass.bind_template();
            // Required because #[gtk::template_callbacks] is on the public `impl InspyrPhotoPage`.
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for InspyrPhotoPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gsettings();
            obj.load_images_from_database();
        }
    }
    impl WidgetImpl for InspyrPhotoPage {}
    impl BinImpl for InspyrPhotoPage {}
}

glib::wrapper! {
    pub struct InspyrPhotoPage(ObjectSubclass<imp::InspyrPhotoPage>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for InspyrPhotoPage {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl InspyrPhotoPage {
    pub fn new() -> Self {
        Self::default()
    }

    #[template_callback]
    fn on_item_setup(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let photo_item = InspyrPhotoItem::new();

        self.bind_property("icon-size", &photo_item, "icon-size")
            .sync_create()
            .build();

        list_item.set_child(Some(&photo_item));
    }

    #[template_callback]
    fn on_item_bind(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let Some(gobj) = list_item.item() else {
            return;
        };
        let Some(item) = gobj.downcast_ref::<InspyrImageRow>() else {
            return;
        };
        let widget = list_item.child().unwrap();
        let photo_item = widget.downcast_ref::<InspyrPhotoItem>().unwrap();

        let path = PathBuf::from(item.path());
        photo_item.load_from_path(&path);
    }

    #[template_callback]
    fn on_item_unbind(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let Some(widget) = list_item.child() else {
            return;
        };
        let photo_item = widget.downcast_ref::<InspyrPhotoItem>().unwrap();
        photo_item.clear_thumbnail();
    }

    fn load_images_from_database(&self) {
        let db = match Database::init() {
            Ok(db) => db,
            Err(e) => {
                glib::g_warning!(LOG_DOMAIN, "Could not open database: {e}");
                return;
            }
        };

        let ops = DatabaseOperations::new(&db);
        let store = gio::ListStore::new::<InspyrImageRow>();

        let mut offset = 0u32;
        loop {
            let opts = ListOptions {
                limit: 1000,
                offset,
            };
            let batch = match ops.list(&opts) {
                Ok(b) => b,
                Err(e) => {
                    glib::g_warning!(LOG_DOMAIN, "Could not list images: {e}");
                    break;
                }
            };
            if batch.is_empty() {
                break;
            }
            let n = batch.len();
            for image in batch {
                store.append(&InspyrImageRow::from_image(&image));
            }
            if n < 1000 {
                break;
            }
            offset = offset.saturating_add(1000);
        }

        let selection = gtk::NoSelection::new(Some(store.clone()));
        self.imp().grid_view.set_model(Some(&selection));
        *self.imp().store.borrow_mut() = Some(store);
    }

    fn setup_gsettings(&self) {
        if !Self::is_schema_installed() {
            glib::g_debug!(
                LOG_DOMAIN,
                "Not binding to settings as schema is not available"
            );
            self.set_icon_size(96);
            return;
        }

        let settings = gio::Settings::new("org.gnome.Inspyr");
        settings.bind("icon-size", self, "icon-size").build();
    }

    fn is_schema_installed() -> bool {
        let Some(source) = gio::SettingsSchemaSource::default() else {
            return false;
        };
        source.lookup("org.gnome.Inspyr", true).is_some()
    }
}
