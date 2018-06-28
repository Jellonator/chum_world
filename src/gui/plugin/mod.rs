use gtk::{self, Widget, Label};
use gtk::prelude::*;
use super::page::{Page, ArchiveFile};
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::CResult;
use std::str;

/// Create a plugin widget for editing TXT files.
/// TXT files have the following format:
/// file_size: u32;
/// data: u8[fize_size];
/// Some TXT files, such as .PYC files, may contain invalid String characters.
fn create_plugin_text(parent: &Rc<RefCell<Page>>, file: &Rc<RefCell<ArchiveFile>>) -> CResult<Widget> {
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
        let _size = slice.read_u32::<BigEndian>()? as usize;
        // If the string can not be converted from utf8, OR if the string
        // contains any null characters, then the text box should not be
        // editable since that would destroy data.
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
    let ftext = Rc::downgrade(&file);
    let ptext = Rc::downgrade(&parent);
    text.get_buffer().unwrap().connect_changed(move |b| {
        let ftext = ftext.upgrade().unwrap();
        let ptext = ptext.upgrade().unwrap();
        let mut vec = Vec::new();
        let text: String = b.get_text(&b.get_start_iter(), &b.get_end_iter(), true).unwrap();
        vec.write_u32::<BigEndian>(text.len() as u32).unwrap();
        vec.write_all(&text.as_ref()).unwrap();
        ftext.borrow_mut().data = vec;
        ptext.borrow_mut().set_need_save(true);
    });

    scroll.add(&text);

    Ok(scroll.upcast::<Widget>())
}

/// Create a widget for editing the given file's data.
/// If the file does not have a pre-determined editor, or an editor for the
/// file could not be created, then a label will be returned.
pub fn create_plugin_for_type(parent: &Rc<RefCell<Page>>, file: &Rc<RefCell<ArchiveFile>>) -> Widget {
    let typestr = &file.borrow().typeid;
    let result = match typestr.as_ref() {
        "TXT" => create_plugin_text(parent, file),
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

