use gtk::prelude::*;
use gtk::{self, FileChooserDialog, FileChooserAction, FileFilter, ResponseType};
use std::path::{Path, PathBuf};
use crc::crc32;
use std::error::Error;
use std::borrow::Borrow;
use std::fs;
use dgc;
use ngc;

/// Complete Chum archive.
/// Contains both a .NGC archive and a .DGC archive.
pub struct ChumArchive {
    pub dgc: dgc::DgcArchive,
    pub ngc: ngc::NgcArchive,
}

/// A Result type that can be any error.
pub type CResult<T> = Result<T, Box<Error>>;

/// Returns Ok(true) if the given folder is not empty
/// Returns Ok(false) if the given folder is empty
/// Returns Err(_) if an error occurs
pub fn is_dir_populated(path: &Path) -> CResult<bool> {
    match fs::read_dir(path)?.next() {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

/// Get the output file name for the given file string and id
pub fn get_file_string(s: &str, id: u32) -> String {
    if let Some(pos) = s.rfind('.') {
        let (left, right) = s.split_at(pos);
        format!("{}[{:8X}]{}", left, id, right)
    } else {
        format!("{}[{:8X}]", s, id)
    }.replace(|c: char| {
        !c.is_alphanumeric() && c != '.'
    } , "_")
}

/// Represents a path that can represent both a NGC and DGC file
#[derive(Clone, PartialEq, Eq)]
pub struct ArchivePathPair {
    pub n: PathBuf,
    pub d: PathBuf,
}

/// Opens any file, doesn't care about file types
pub fn open_any<W>(base_path: &Path, prompt: &str, parent: &W, action: FileChooserAction)
-> Option<PathBuf>
where W: gtk::IsA<gtk::Window> {
    let btn: &str = match action {
        FileChooserAction::Open | FileChooserAction::SelectFolder => &gtk::STOCK_OPEN,
        _ => &gtk::STOCK_SAVE,
    };
    let dialog = FileChooserDialog::with_buttons(
        Some(prompt),  Some(parent), action,
        &[(&gtk::STOCK_CANCEL, ResponseType::Cancel), (btn, ResponseType::Accept)]);
    match action {
        FileChooserAction::Open | FileChooserAction::Save => {
            let file_filter = FileFilter::new();
            file_filter.add_pattern("*.*");
            gtk::FileFilterExt::set_name(&file_filter, "Any file");
            dialog.add_filter(&file_filter);
        }
        _ => {}
    }
    dialog.set_current_folder(base_path);

    let result = match dialog.run().into() {
        ResponseType::Accept => dialog.get_filename(),
        _ => None
    };

    dialog.destroy();

    result
}

/// Open a DGC file and construct a DGC/NGC file path pair
pub fn open_gc<W>(base_path: &Path, parent: &W, action: FileChooserAction)
-> Option<ArchivePathPair>
where W: gtk::IsA<gtk::Window> {
    let btn: &str = match action {
        FileChooserAction::Open | FileChooserAction::SelectFolder => &gtk::STOCK_OPEN,
        _ => &gtk::STOCK_SAVE,
    };
    let dialog = FileChooserDialog::with_buttons(
        Some("Open File"),  Some(parent), action,
        &[(&gtk::STOCK_CANCEL, ResponseType::Cancel), (btn, ResponseType::Accept)]);
    match action {
        FileChooserAction::Open | FileChooserAction::Save => {
            let file_filter = FileFilter::new();
            file_filter.add_pattern("*.DGC");
            gtk::FileFilterExt::set_name(&file_filter, "DGC files");
            dialog.add_filter(&file_filter);
        }
        _ => {}
    }
    dialog.set_current_folder(base_path);

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

/// Confirm that the user wants to perform an action.
/// Returns true if the action should be performed.
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

/// Handle the given result and show an error window if it is a Result::Err
pub fn handle_result<W>(err: CResult<()>, base_msg: &str, parent: &W)
where W: gtk::IsA<gtk::Window> {
    match err {
        Ok(_) => {},
        Err(ref err) => show_error(err.borrow(), base_msg, parent),
    }
}

/// Show an error to the user
pub fn show_error<W>(err: &Error, base_msg: &str, parent: &W)
where W: gtk::IsA<gtk::Window> {
    let flags = gtk::DialogFlags::DESTROY_WITH_PARENT;
    let dialog = gtk::MessageDialog::new(
        Some(parent), flags, gtk::MessageType::Error,
        gtk::ButtonsType::Ok, &format!("{}:\n{}", base_msg, err.description()));
    dialog.run();
    dialog.destroy();
}

/// Hash the given name using the crc32 IEEE algorithm.
pub fn hash_name(name: &str) -> i32 {
    crc32::checksum_ieee(&name.as_bytes()) as i32
}
