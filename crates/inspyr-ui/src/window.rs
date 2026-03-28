/* window.rs
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
use gettextrs::gettext;
use gtk::prelude::IsA;
use gtk::{gio, glib, CompositeTemplate, TemplateChild};

use crate::photo_page::InspyrPhotoPage;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Inspyr/window.ui")]
    pub struct InspyrWindow {
        #[template_child]
        pub stack_main: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub photo_page: TemplateChild<InspyrPhotoPage>,
        #[template_child]
        pub search_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub search_bar: TemplateChild<gtk::SearchBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InspyrWindow {
        const NAME: &'static str = "InspyrWindow";
        type Type = super::InspyrWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            InspyrPhotoPage::static_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for InspyrWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_search_header();
        }
    }
    impl WidgetImpl for InspyrWindow {}
    impl WindowImpl for InspyrWindow {}
    impl ApplicationWindowImpl for InspyrWindow {}
    impl AdwApplicationWindowImpl for InspyrWindow {}
}

glib::wrapper! {
    pub struct InspyrWindow(ObjectSubclass<imp::InspyrWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl InspyrWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_search_header(&self) {
        let imp = self.imp();
        let photo_view_stack = imp.photo_page.view_stack();
        photo_view_stack.connect_visible_child_name_notify(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_| win.sync_search_button_with_photo_viewer()
        ));
        imp.stack_main
            .connect_visible_child_name_notify(glib::clone!(
                #[weak(rename_to = win)]
                self,
                move |_| win.sync_search_button_with_photo_viewer()
            ));
        imp.search_button.connect_toggled(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |btn| {
                if !win.main_header_shows_photo_back() {
                    win.imp().search_bar.set_search_mode(btn.is_active());
                    return;
                }
                if btn.is_active() {
                    win.imp().photo_page.close_photo_viewer();
                    btn.set_active(false);
                }
            }
        ));
        self.sync_search_button_with_photo_viewer();
    }

    fn main_header_shows_photo_back(&self) -> bool {
        let imp = self.imp();
        if imp.stack_main.visible_child_name().as_deref() != Some("photos") {
            return false;
        }
        imp.photo_page.view_stack().visible_child_name().as_deref() == Some("photo_view")
    }

    fn sync_search_button_with_photo_viewer(&self) {
        let imp = self.imp();
        let btn = imp.search_button.get();
        let bar = imp.search_bar.get();
        if self.main_header_shows_photo_back() {
            btn.set_icon_name("go-previous-symbolic");
            btn.set_tooltip_text(Some(&gettext("Back to grid")));
            bar.set_search_mode(false);
            btn.set_active(false);
        } else {
            btn.set_icon_name("edit-find-symbolic");
            btn.set_tooltip_text(Some(&gettext("Search")));
        }
    }
}
