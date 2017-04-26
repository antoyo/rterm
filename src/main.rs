/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#![feature(proc_macro)]

extern crate gdk;
extern crate glib;
extern crate gobject_sys;
extern crate gtk;
extern crate libc;
extern crate pango_sys;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate vte;

mod gobject;

use std::env;

use gdk::{ModifierType, CONTROL_MASK};
use gdk::enums::key::{Escape, Key, n, p, slash};
use glib::translate::ToGlib;
use gtk::{
    CssProvider,
    Entry,
    EntryExt,
    Inhibit,
    OrientableExt,
    Settings,
    WidgetExt,
    WindowExt,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm::gtk_ext::BoxExtManual;
use relm_attributes::widget;
use vte::Regex;

use gobject::ObjectExtManual;

use Msg::*;

const PCRE2_CASELESS: u32 = 0x8;
const PCRE2_MULTILINE: u32 = 0x0400;
const PCRE2_NO_UTF8_CHECK: u32 = 0x80000;
const PCRE2_UTF: u32 = 0x40000000;

#[derive(Clone)]
pub struct Model {
    entry_text: &'static str,
    search_entry_visible: bool,
    title: String,
    urgent: bool,
}

#[derive(Msg)]
pub enum Msg {
    Bell,
    FocusIn,
    KeyPress((Key, ModifierType)),
    Quit,
    Search(String),
    TitleChanged(String),
}

#[widget]
impl Widget for Win {
    fn init_view(&self) {
        set_dark_theme();
        let directory = env::home_dir();
        let shell = env::var_os("SHELL").unwrap();
        self.terminal.spawn_async(directory, &[shell.to_str().unwrap()], &[]);
        adjust_entry_look(&self.search_entry);
    }

    fn model() -> Model {
        Model {
            entry_text: "",
            search_entry_visible: false,
            title: String::new(),
            urgent: false,
        }
    }

    #[allow(non_upper_case_globals)]
    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Bell => model.urgent = true,
            FocusIn => model.urgent = false,
            KeyPress((key, modifier)) => {
                if modifier & CONTROL_MASK == CONTROL_MASK {
                    match key {
                        n => {
                            self.terminal.search_find_previous();
                        },
                        p => {
                            self.terminal.search_find_next();
                        },
                        slash => {
                            model.search_entry_visible = true;
                            self.search_entry.grab_focus();
                        },
                        _ => (),
                    }
                }
                else {
                    match key {
                        Escape => {
                            model.search_entry_visible = false;
                            model.entry_text = "";
                            self.terminal.grab_focus();
                            self.search(None);
                        },
                        _ => (),
                    }
                }
            },
            Quit => gtk::main_quit(), // TODO: confirm before leaving if running a command?
            Search(pattern) => {
                if self.search(Some(pattern)) {
                    self.terminal.search_find_previous();
                }
            },
            TitleChanged(title) => model.title = title,
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="terminal"]
                vte::Terminal {
                    packing: {
                        expand: true,
                    },
                    font_scale: 1.2,
                    hexpand: true,
                    bell => Bell,
                    child_exited(_, _) => Quit,
                    window_title_changed(terminal) => TitleChanged(
                        terminal.get_window_title().unwrap_or_else(String::new)),
                },
                #[name="search_entry"]
                gtk::Entry {
                    activate(entry) => Search(entry.get_text().unwrap_or_else(String::new)),
                    name: "rterm-search-input",
                    text: model.entry_text,
                    visible: model.search_entry_visible,
                },
            },
            title: &model.title,
            urgency_hint: model.urgent,
            delete_event(_, _) => (Quit, Inhibit(false)),
            focus_in_event(_, _) => (FocusIn, Inhibit(false)),
            key_press_event(_, key) => (KeyPress((key.get_keyval(), key.get_state())), Inhibit(false)),
        }
    }
}

impl Win {
    fn search(&self, pattern: Option<String>) -> bool {
        if let Some(pattern) = pattern {
            if let Ok(regex) = Regex::new_for_search(&pattern, PCRE2_CASELESS | PCRE2_MULTILINE | PCRE2_UTF | PCRE2_NO_UTF8_CHECK) {
                self.terminal.search_set_regex(Some(&regex), 0);
                return true;
            }
        }
        else {
            // FIXME: not working.
            self.terminal.search_set_regex(None, 0);
        }
        false
    }
}

fn adjust_entry_look(entry: &Entry) {
    let style_context = entry.get_style_context().unwrap();
    let style = include_str!("../style/command-input.css");
    let provider = CssProvider::new();
    provider.load_from_data(style).unwrap();
    style_context.add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
}

fn set_dark_theme() {
    let settings = Settings::get_default().unwrap();
    settings.set_data("gtk-application-prefer-dark-theme", true.to_glib());
}

fn main() {
    relm::run::<Win>().unwrap();
}
