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

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gio, glib, CompositeTemplate};
use std::cell::Cell;
 
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
     }
 
     #[glib::object_subclass]
     impl ObjectSubclass for InspyrPhotoPage {
         const NAME: &'static str = "InspyrPhotoPage";
         type Type = super::InspyrPhotoPage;
         type ParentType = adw::Bin;
 
         fn class_init(klass: &mut Self::Class) {
             klass.bind_template();
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