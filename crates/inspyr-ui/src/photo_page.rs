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
use gtk::gio::prelude::ListModelExt;
use gtk::{gio, glib, CompositeTemplate, ListScrollFlags};
use inspyr_database::{Database, DatabaseOperations, ListOptions};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;

const LOG_DOMAIN: &str = "InspyrPhotoPage";
/// Square strip cell edge length; `GtkPicture` uses cover so pixels fill the selection outline.
const STRIP_THUMB_SIZE: u32 = 32;

/// Square strip cell; no outer margins so list selection/focus matches the image bounds.
fn strip_cell_extent() -> i32 {
    STRIP_THUMB_SIZE as i32
}

/// Scrolled strip height: cell row plus space for the horizontal scrollbar when shown.
fn strip_scrolled_height_request() -> i32 {
    strip_cell_extent() + (STRIP_THUMB_SIZE as i32 / 4).max(12)
}

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/org/gnome/Inspyr/photo-page.ui")]
    #[properties(wrapper_type = super::InspyrPhotoPage)]
    pub struct InspyrPhotoPage {
        #[template_child]
        pub view_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub grid_view: TemplateChild<gtk::GridView>,
        #[template_child]
        pub single_selection: TemplateChild<gtk::SingleSelection>,
        #[template_child]
        pub viewer_root: TemplateChild<gtk::Box>,
        #[template_child]
        pub viewer_picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub viewer_prev_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub viewer_next_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub strip_list: TemplateChild<gtk::ListView>,
        #[template_child]
        pub strip_scrolled: TemplateChild<gtk::ScrolledWindow>,

        #[property(get, set)]
        icon_size: Cell<u32>,

        /// Keeps the list model alive while the grid is shown.
        pub store: RefCell<Option<gio::ListStore>>,
        /// Index into `store` when `photo_view` is visible.
        pub viewer_index: Cell<u32>,
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
            obj.apply_strip_shell_geometry();
            obj.load_images_from_database();
            obj.setup_photo_viewer_interaction();
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

    fn apply_strip_shell_geometry(&self) {
        self.imp()
            .strip_scrolled
            .set_height_request(strip_scrolled_height_request());
    }

    #[template_callback]
    fn on_item_setup(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let photo_item = InspyrPhotoItem::new();

        self.bind_property("icon-size", &photo_item, "icon-size")
            .sync_create()
            .build();

        let s = self.icon_size();
        if s > 0 {
            photo_item.set_size_request(s as i32, s as i32);
        }

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

    #[template_callback]
    fn on_activate(&self, position: u32, _grid_view: gtk::GridView) {
        self.open_photo_viewer_at(position);
    }

    #[template_callback]
    fn on_strip_activate(&self, position: u32, _list_view: gtk::ListView) {
        self.open_photo_viewer_at(position);
    }

    #[template_callback]
    fn on_strip_item_setup(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let cell = strip_cell_extent();
        let pic = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::Cover)
            .can_shrink(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .hexpand(true)
            .vexpand(true)
            .build();
        pic.set_size_request(cell, cell);
        list_item.set_child(Some(&pic));
    }

    #[template_callback]
    fn on_strip_item_bind(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let Some(gobj) = list_item.item() else {
            return;
        };
        let Some(item) = gobj.downcast_ref::<InspyrImageRow>() else {
            return;
        };
        let widget = list_item.child().unwrap();
        let pic = widget.downcast_ref::<gtk::Picture>().unwrap();

        let path = PathBuf::from(item.path());
        if path.exists() {
            pic.set_file(Some(&gio::File::for_path(&path)));
        } else {
            pic.set_file(None::<&gio::File>);
        }
    }

    #[template_callback]
    fn on_strip_item_unbind(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let Some(widget) = list_item.child() else {
            return;
        };
        let pic = widget.downcast_ref::<gtk::Picture>().unwrap();
        pic.set_file(None::<&gio::File>);
    }

    #[template_callback]
    fn on_viewer_prev_clicked(&self, _button: gtk::Button) {
        self.step_viewer(-1);
    }

    #[template_callback]
    fn on_viewer_next_clicked(&self, _button: gtk::Button) {
        self.step_viewer(1);
    }

    fn setup_photo_viewer_interaction(&self) {
        let imp = self.imp();
        let swipe = gtk::GestureSwipe::new();
        swipe.set_propagation_phase(gtk::PropagationPhase::Capture);
        swipe.connect_swipe(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, vx, _vy| {
                const THRESH: f64 = 500.0;
                if vx > THRESH {
                    page.step_viewer(-1);
                } else if vx < -THRESH {
                    page.step_viewer(1);
                }
            }
        ));
        imp.viewer_picture.add_controller(swipe);

        let keys = gtk::EventControllerKey::new();
        keys.connect_key_pressed(glib::clone!(
            #[weak(rename_to = page)]
            self,
            #[upgrade_or_else]
            || glib::Propagation::Proceed,
            move |_ec, key, _, _| {
                if key == gtk::gdk::Key::Escape {
                    page.close_photo_viewer();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        imp.viewer_root.add_controller(keys);
    }

    fn open_photo_viewer_at(&self, index: u32) {
        let imp = self.imp();
        let Some(store) = imp.store.borrow().clone() else {
            return;
        };
        if store.n_items() == 0 || index >= store.n_items() {
            return;
        }
        imp.viewer_index.set(index);
        imp.single_selection.set_selected(index);
        self.reload_viewer_picture();
        self.update_viewer_nav_buttons();
        imp.view_stack
            .set_visible_child_full("photo_view", gtk::StackTransitionType::Crossfade);
        imp.viewer_root.grab_focus();
        self.scroll_strip_to_index(index);
    }

    pub fn close_photo_viewer(&self) {
        let imp = self.imp();
        imp.viewer_picture.set_file(None::<&gio::File>);
        imp.view_stack
            .set_visible_child_full("thumbnail_view", gtk::StackTransitionType::Crossfade);
    }

    pub fn view_stack(&self) -> gtk::Stack {
        self.imp().view_stack.get()
    }

    fn step_viewer(&self, delta: i32) {
        let imp = self.imp();
        let Some(store) = imp.store.borrow().clone() else {
            return;
        };
        let n = store.n_items();
        if n == 0 {
            return;
        }
        let i = imp.viewer_index.get() as i64 + delta as i64;
        if i < 0 || i >= n as i64 {
            return;
        }
        let new_i = i as u32;
        imp.viewer_index.set(new_i);
        imp.single_selection.set_selected(new_i);
        self.reload_viewer_picture();
        self.update_viewer_nav_buttons();
        self.scroll_strip_to_index(new_i);
    }

    fn scroll_strip_to_index(&self, index: u32) {
        let list = self.imp().strip_list.get();
        list.scroll_to(index, ListScrollFlags::FOCUS, None);

        let page = self.clone();
        glib::idle_add_local(move || {
            page.center_strip_thumb(index);
            glib::ControlFlow::Break
        });
    }

    /// Places the thumbnail for `index` at the horizontal center of the strip viewport.
    fn center_strip_thumb(&self, index: u32) {
        let imp = self.imp();
        let Some(store) = imp.store.borrow().clone() else {
            return;
        };
        let n = store.n_items();
        if n == 0 || index >= n {
            return;
        }

        let list = imp.strip_list.get();
        let Some(hadj) = list.hadjustment() else {
            return;
        };
        let lower = hadj.lower();
        let upper = hadj.upper();
        let page_size = hadj.page_size();

        if page_size <= f64::EPSILON {
            return;
        }

        // `upper - lower` spans the full content width; each cell gets an equal share.
        let span = upper - lower;
        if span <= page_size {
            hadj.set_value(lower);
            return;
        }

        let cell = span / f64::from(n);
        let center = f64::from(index) * cell + cell / 2.0;
        let max_val = (upper - page_size).max(lower);
        let value = (center - page_size / 2.0).clamp(lower, max_val);
        hadj.set_value(value);
    }

    fn reload_viewer_picture(&self) {
        let imp = self.imp();
        let picture = imp.viewer_picture.get();
        let Some(store) = imp.store.borrow().clone() else {
            picture.set_file(None::<&gio::File>);
            return;
        };
        let idx = imp.viewer_index.get();
        let Some(obj) = store.item(idx) else {
            picture.set_file(None::<&gio::File>);
            return;
        };
        let Some(row) = obj.downcast_ref::<InspyrImageRow>() else {
            picture.set_file(None::<&gio::File>);
            return;
        };
        let path = PathBuf::from(row.path());
        if path.exists() {
            picture.set_file(Some(&gio::File::for_path(&path)));
        } else {
            picture.set_file(None::<&gio::File>);
        }
    }

    fn update_viewer_nav_buttons(&self) {
        let imp = self.imp();
        let Some(store) = imp.store.borrow().clone() else {
            return;
        };
        let n = store.n_items();
        let i = imp.viewer_index.get();
        imp.viewer_prev_button.set_sensitive(i > 0);
        imp.viewer_next_button.set_sensitive(i + 1 < n);
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

        self.imp().single_selection.set_model(Some(&store));
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
