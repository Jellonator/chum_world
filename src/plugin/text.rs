use gtk::{self, Widget};
use gtk::prelude::*;
use gui::page::{Page, ArchiveFile};
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::CResult;
use std::str;
use super::FilePlugin;

/// A plugin used for handling text files with the following format:
/// size: u32;
/// data: char[size];
pub struct FilePluginLengthText;

impl FilePlugin for FilePluginLengthText {
    fn import_data(&self, input: &mut Read, output: &mut Write) -> CResult<()> {
        let mut data: Vec<u8> = Vec::new();
        input.read_to_end(&mut data)?;
        output.write_u32::<BigEndian>(data.len() as u32)?;
        output.write_all(&data)?;
        Ok(())
    }

    fn export_data(&self, input: &mut Read, output: &mut Write) -> CResult<()> {
        let _size = input.read_u32::<BigEndian>()? as usize;
        io::copy(input, output)?;
        Ok(())
    }

    fn create_editor(&self, parent: &Rc<RefCell<Page>>, file: &Rc<RefCell<ArchiveFile>>) -> CResult<Widget> {
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

    fn get_plugin_string(&self) -> &'static str {
        "length-text"
    }
}
