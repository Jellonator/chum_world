use gtk::{self, Widget, Label};
use gtk::prelude::*;
use super::page::ArchiveFile;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::CResult;
use std::error::Error;
use std::str;

fn create_plugin_text(file: &Rc<RefCell<ArchiveFile>>) -> CResult<Widget> {
    let scroll = gtk::ScrolledWindow::new(None, None);
    scroll.set_margin_left(4);
    scroll.set_margin_right(4);
    scroll.set_margin_top(4);
    scroll.set_margin_bottom(4);
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    let text = gtk::TextView::new();
    text.set_editable(true);
    {
        let bfile = file.borrow();
        let mut slice = bfile.data.as_slice();
        let size = slice.read_u32::<BigEndian>()? as usize;
        println!("{} {}", slice.len(), size);
        let mut s: String = if let Ok(s) = str::from_utf8(&slice) {
            s.to_owned()
        } else {
            text.set_editable(false);
            String::from_utf8_lossy(&slice).to_string()
        };
        if s.contains('\x00') {
            s = s.replace('\x00', "\u{FFFD}");
            text.set_editable(false);
        }
        text.get_buffer().unwrap().set_text(&s);
    }
    let ftext = file.clone();
    text.get_buffer().unwrap().connect_changed(move |b| {
        let mut vec = Vec::new();
        let text: String = b.get_text(&b.get_start_iter(), &b.get_end_iter(), true).unwrap();
        vec.write_u32::<BigEndian>(text.len() as u32).unwrap();
        vec.write_all(&text.as_ref()).unwrap();
        ftext.borrow_mut().data = vec;
    });

    scroll.add(&text);

    Ok(scroll.upcast::<Widget>())
}

pub fn create_plugin_for_type(file: &Rc<RefCell<ArchiveFile>>) -> Widget {
    let typestr = &file.borrow().typeid;
    println!("{}", typestr);
    let result = match typestr.as_ref() {
        "TXT" => create_plugin_text(file),
        _ => {
            let ret = Label::new(format!("No editor for type {} exists.", typestr).as_str());
            Ok(ret.upcast::<Widget>())
        }
    };
    match result {
        Ok(widget) => widget,
        Err(err) => {
            let ret = Label::new(format!("Error opening file:\n{}", err.description()).as_str());
            ret.upcast::<Widget>()
        }
    }
}

