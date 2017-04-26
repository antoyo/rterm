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

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate vte;

use std::env;

use gtk::{Inhibit, WidgetExt, WindowExt};
use relm::Widget;
use relm_attributes::widget;

use Msg::*;

#[derive(Clone)]
pub struct Model {
    title: String,
    urgent: bool,
}

#[derive(Msg)]
pub enum Msg {
    Bell,
    FocusIn,
    Quit,
    TitleChanged(String),
}

#[widget]
impl Widget for Win {
    fn init_view(&self) {
        let directory = env::home_dir();
        let shell = env::var_os("SHELL").unwrap();
        self.terminal.spawn_async(directory, &[shell.to_str().unwrap()], &[]);
    }

    fn model() -> Model {
        Model {
            title: String::new(),
            urgent: false,
        }
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Bell => model.urgent = true,
            FocusIn => model.urgent = false,
            Quit => gtk::main_quit(), // TODO: confirm before leaving if running a command?
            TitleChanged(title) => model.title = title,
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                #[name="terminal"]
                vte::Terminal {
                    font_scale: 1.2,
                    hexpand: true,
                    bell => Bell,
                    child_exited(_, _) => Quit,
                    window_title_changed(terminal) => TitleChanged(
                        terminal.get_window_title().unwrap_or_else(String::new)),
                }
            },
            title: &model.title,
            urgency_hint: model.urgent,
            delete_event(_, _) => (Quit, Inhibit(false)),
            focus_in_event(_, _) => (FocusIn, Inhibit(false)),
        }
    }
}

fn main() {
    relm::run::<Win>().unwrap();
}
