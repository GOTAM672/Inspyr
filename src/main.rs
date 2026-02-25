/* main.rs
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

mod application;
mod config;
mod window;

use self::application::InspyrApplication;
use self::window::InspyrWindow;

use config::{GETTEXT_PACKAGE, LOCALEDIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::{gdk, gio, glib};
use gtk::prelude::*;

fn main() -> glib::ExitCode {
    let _ = gtk::init();
    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Load resources
    gio::resources_register_include_impl(include_bytes!(concat!(
        env!("INSPYR_RESOURCE_DIR"),
        "/",
        "inspyr.gresource"
    )))
    .expect("Failed to register inspyr resources.");

    let display = gdk::Display::default().unwrap();
    let icon_theme = gtk::IconTheme::for_display(&display);
    gtk::IconTheme::add_resource_path(&icon_theme, "/org/gnome/Inspyr/icons");

    // Create a new GtkApplication. The application manages our main loop,
    // application windows, integration with the window manager/compositor, and
    // desktop features such as file opening and single-instance applications.
    let app = InspyrApplication::new("org.gnome.Inspyr", &gio::ApplicationFlags::empty());

    // Run the application. This function will block until the application
    // exits. Upon return, we have our exit code to return to the shell. (This
    // is the code you see when you do `echo $?` after running a command in a
    // terminal.
    app.run()
}
