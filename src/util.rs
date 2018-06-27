use gtk::prelude::*;
use gtk::{self, FileChooserDialog, FileChooserAction, FileFilter, ResponseType};
use std::path::{Path, PathBuf};
use crc::crc32;

pub fn get_file_string(s: &str, id: u32) -> String {
    if let Some(pos) = s.rfind('.') {
        let (left, right) = s.split_at(pos);
        format!("{}{:8X}{}", left, id, right)
    } else {
        format!("{}{:8X}", s, id)
    }.replace(|c: char| {
        !c.is_alphanumeric() && c != '.'
    } , "_")
}

#[derive(Clone)]
pub struct ArchivePathPair {
    pub n: PathBuf,
    pub d: PathBuf,
}

pub fn open_any<W>(base_path: &Path, prompt: &str, parent: &W, action: FileChooserAction) 
-> Option<PathBuf>
where W: gtk::IsA<gtk::Window> {
    let dialog = FileChooserDialog::with_buttons(
        Some(prompt),  Some(parent), action,
        &[(&gtk::STOCK_CANCEL, ResponseType::Cancel), (&gtk::STOCK_OPEN, ResponseType::Accept)]);
    let file_filter = FileFilter::new();
    file_filter.add_pattern("*.*");
    gtk::FileFilterExt::set_name(&file_filter, "Any file");
    dialog.add_filter(&file_filter);
    dialog.set_current_folder(base_path);

    let result = match dialog.run().into() { 
        ResponseType::Accept => dialog.get_filename(),
        _ => None
    };

    dialog.destroy();

    result
}

pub fn open_gc<W>(base_path: &Path, parent: &W, action: FileChooserAction) 
-> Option<ArchivePathPair>
where W: gtk::IsA<gtk::Window> {
    let dialog = FileChooserDialog::with_buttons(
        Some("Open File"),  Some(parent), action,
        &[(&gtk::STOCK_CANCEL, ResponseType::Cancel), (&gtk::STOCK_OPEN, ResponseType::Accept)]);
    let file_filter = FileFilter::new();
    file_filter.add_pattern("*.DGC");
    gtk::FileFilterExt::set_name(&file_filter, "DGC files");
    dialog.add_filter(&file_filter);
    //dialog.set_current_folder(base_path);

    let result = match dialog.run().into() { 
        ResponseType::Accept => dialog.get_filename().map(|dname| {
            let dpath: PathBuf = dname.into();
            let npath: PathBuf = dpath.with_extension("NGC");
            ArchivePathPair {
                n: npath,
                d: dpath,
            }
        }),
        _ => None
    };

    dialog.destroy();

    result
}

pub fn ask_confirmation<W>(parent: &W, msg: &str) -> bool
where W: gtk::IsA<gtk::Window> {
    let flags = gtk::DialogFlags::DESTROY_WITH_PARENT;
    let dialog = gtk::MessageDialog::new(
        Some(parent), flags, gtk::MessageType::Warning, 
        gtk::ButtonsType::YesNo, msg);
    let value = dialog.run();
    dialog.destroy();
    match value.into() {
        gtk::ResponseType::Yes => true,
        _ => false,
    }
}

pub fn hash_name(name: &str) -> i32 {
    crc32::checksum_ieee(&name.as_bytes()) as i32
}
