use gtk::{self, Container, Label, ScrolledWindow, Paned, ListBox};
use gtk::prelude::*;
use ::CResult;
use util::{self, ArchivePathPair};
use ngc::NgcArchive;
use dgc::{DgcArchive, DgcFile};
use std::fs::File;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::editor;
use super::app::Application;
use plugin;

/// Single DGC file, but with IDs replaced with names.
pub struct ArchiveFile {
    pub data: Vec<u8>,
    pub name: String,
    pub typeid: String,
    pub subtypeid: String,
}

/// Represents a DGC/NGC archive pair, except files include their name and type information as
/// strings. Files are also given the Rc+RefCell pattern so that they can be shared by editors.
pub struct Archive {
    pub files: Vec<Rc<RefCell<ArchiveFile>>>,
    pub header: String,
}

/// Represents a single archive file page.
pub struct Page {
    pub paths: ArchivePathPair,
    pub container: Container,
    pub label: Label,
    pub archive: Archive,
    pub list: ListBox,
    pub parent: Weak<RefCell<Application>>,
    pub tool: gtk::Box,
    pub need_save: bool,
    pub plugin_manager: plugin::PluginManager,
    stop_recurse: bool,
}

impl Archive {
    /// Create an archive from a DGC/NGC file pair
    pub fn from_archives(data: DgcArchive, names: NgcArchive) -> Archive {
        Archive {
            header: String::from_utf8_lossy(&data.header.legal_notice).into(),
            files: data.data.into_iter().flat_map(|chunk| chunk.data.into_iter()).map(|f| {
                let name = names.names[&f.id1].clone();
                let mut subtypeid = names.names[&f.id2].clone();
                if name == subtypeid {
                    subtypeid = "".to_string();
                }
                Rc::new(RefCell::new(ArchiveFile {
                    name: name,
                    typeid: names.names[&f.type_id].clone(),
                    subtypeid: subtypeid,
                    data: f.data,
                }))
            }).collect(),
        }
    }

    /// Create a DGC/NGC file pair from this archive
    pub fn into_archives(&self) -> (DgcArchive, NgcArchive) {
        let mut dgc = DgcArchive::new(&self.header, 0);
        let mut ngc = NgcArchive::new();
        for file in &self.files {
            let file = file.borrow();
            let id1: i32 = util::hash_name(&file.name);
            let (id2, subtypeid): (i32, String) = if file.subtypeid == "" {
                (id1, file.name.clone())
            } else {
                (util::hash_name(&file.subtypeid), file.subtypeid.clone())
            };
            let type_id: i32 = util::hash_name(&file.typeid);
            dgc.add_file(DgcFile {
                data: file.data.clone(),
                id1: id1,
                id2: id2,
                type_id: type_id,
            });
            ngc.names.insert(id1, file.name.clone());
            ngc.names.insert(id2, subtypeid);
            ngc.names.insert(type_id, file.typeid.clone());
        }
        (dgc, ngc)
    }

    /// Sort all of the files in this archive by name
    pub fn sort_files(&mut self) {
        self.files.sort_by(|a, b| {
            let a = a.borrow();
            let b = b.borrow();
            if a.typeid != b.typeid {
                a.typeid.cmp(&b.typeid)
            } else if a.subtypeid != b.subtypeid {
                a.subtypeid.cmp(&b.subtypeid)
            } else {
                a.name.cmp(&b.name)
            }
        });
    }

    /// Return true if the file exists
    pub fn exists(&self, name: &str) -> bool{
        for f in &self.files {
            if f.borrow().name == name {
                return true;
            }
        }
        false
    }

    /// Find the file in the archive
    pub fn find(&self, name: &str) -> Option<usize> {
        for i in 0..self.files.len() {
            if self.files[i].borrow().name == name {
                return Some(i);
            }
        }
        return None;
    }

    /// Add the file to this archive
    /// Returns the file that was replaced by this file
    pub fn add(&mut self, file: ArchiveFile) -> Option<ArchiveFile> {
        if let Some(i) = self.find(&file.name) {
            Some(self.files[i].replace(file))
        } else {
            self.files.push(Rc::new(RefCell::new(file)));
            None
        }
    }
}

impl Page {
    /// Set whether or not this file needs to be saved
    pub fn set_need_save(&mut self, new_need_save: bool) {
        if new_need_save != self.need_save {
            self.label.set_text(&format!("{}{}",
                self.paths.d.file_name().unwrap().to_str().unwrap(),
                match new_need_save {
                    true => "*",
                    false => " ",
                }
            ));
        }
        self.need_save = new_need_save;
    }

    /// Create a new archive page
    pub fn new(parent: &Rc<RefCell<Application>>, paths: ArchivePathPair) -> CResult<Rc<RefCell<Page>>> {
        let label = Label::new(paths.d.file_name().unwrap().to_str().unwrap());
        // load files
        let mut name_file = File::open(&paths.n)?;
        let mut data_file = File::open(&paths.d)?;
        let dgca = DgcArchive::read_from(&mut data_file)?;
        let ngca = NgcArchive::read_from(&mut name_file)?;
        // create pane
        let pane = Paned::new(gtk::Orientation::Horizontal);
        let list_scroll = ScrolledWindow::new(None, None);
        list_scroll.set_size_request(64, 64);
        let list = ListBox::new();
        list_scroll.add(&list);
        pane.pack1(&list_scroll, true, false);
        let tool = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        pane.pack2(&tool, true, false);
        tool.set_size_request(64, 64);
        tool.set_property_expand(true);
        // create page
        let page = Rc::new(RefCell::new(Page {
            paths: paths,
            container: pane.clone().upcast::<Container>(),
            label: label.clone(),
            list: list.clone(),
            archive: Archive::from_archives(dgca, ngca),
            parent: Rc::downgrade(parent),
            tool: tool,
            need_save: true,
            plugin_manager: plugin::PluginManager::new(),
            stop_recurse: false,
        }));
        Page::update_file_list(&page);
        page.borrow_mut().set_need_save(false);
        // Add callback for row selection
        let lspage = Rc::downgrade(&page);
        list.connect_row_selected(move |_, _| {
            let lspage = lspage.upgrade().unwrap();
            if !lspage.borrow().stop_recurse {
                lspage.borrow_mut().stop_recurse = true;
                let file = lspage.borrow().get_active_file();
                Page::soft_update_file_list(&lspage);
                Page::set_active_file(&lspage, file.as_ref());
                Page::reset_file_editor(&lspage);
                lspage.borrow_mut().stop_recurse = false;
            }
        });
        Ok(page)
    }

    /// Reset editor for file
    pub fn reset_file_editor(page: &Rc<RefCell<Page>>) {
        let row = page.borrow().list.get_selected_row();
        // Remove existing editor
        for child in &page.borrow().tool.get_children() {
            // page.borrow().tool.remove(child);
            child.destroy();
        }
        // Create editor (or empty label if no files are selected)
        if let Some(row) = row {
            let file = page.borrow().archive.files[row.get_index() as usize].clone();
            let editor = editor::construct_editor(page.clone(), file, row.get_index());
            page.borrow().tool.add(&editor);
        } else {
            let placeholder = gtk::Label::new("");
            page.borrow().tool.add(&placeholder);
        }
        page.borrow().tool.show_all();
    }

    /// Completely update the file list
    pub fn update_file_list(page: &Rc<RefCell<Page>>) {
        page.borrow_mut().archive.sort_files();
        // Page::set_active_file(page, None);
        let list = page.borrow().list.clone();
        page.borrow_mut().stop_recurse = true;
        list.unselect_all();
        page.borrow_mut().stop_recurse = false;
        // Remove all files
        let page = page.borrow_mut();
        for w in &list.get_children() {
            // page.list.remove(w);
            w.destroy();
        }
        // Generate new, better files
        for file in &page.archive.files {
            let file = file.borrow();
            let row_label = Label::new(file.name.as_str());
            row_label.set_justify(gtk::Justification::Left);
            row_label.set_halign(gtk::Align::Start);
            list.add(&row_label);
        }
        list.show_all();
    }

    /// Only update label names in the file list
    pub fn soft_update_file_list(page: &Rc<RefCell<Page>>) {
        let mut page = page.borrow_mut();
        page.archive.sort_files();
        // Generate new, better files
        for i in 0..page.archive.files.len() {
            let file = page.archive.files[i].borrow();
            // let row_label = Label::new(file.name.as_str());
            let row_label = page.list.get_row_at_index(i as i32)
                .unwrap().get_children()[0].clone().downcast::<Label>().unwrap();
            row_label.set_text(file.name.as_str());
        }
        page.list.show_all();
    }

    pub fn get_active_file(&self) -> Option<Rc<RefCell<ArchiveFile>>> {
        let row = self.list.get_selected_row();
        row.map(|row| self.archive.files[row.get_index() as usize].clone())
    }

    pub fn set_active_file(page: &Rc<RefCell<Page>>, file: Option<&Rc<RefCell<ArchiveFile>>>) {
        let row = {
            let page = page.borrow();
            let rowid = file.and_then(|file| page.archive.find(&file.borrow().name));
            rowid.and_then(|id| page.list.get_row_at_index(id as i32))
        };
        let list = page.borrow().list.clone();
        list.select_row(row.as_ref());
    }

    /// Set the name of the given file in the file list
    pub fn set_file_name(&self, id: i32, name: &str) {
        let list = self.list.clone();
        let widget = list.get_row_at_index(id).unwrap().get_children().get(0).unwrap().clone();//.dynamic_cast::<Label>();//.unwrap().set_text(name);
        widget.downcast::<Label>().unwrap().set_text(name);
    }

    /// Save the archive
    pub fn save(&mut self) -> CResult<()> {
        let (dgc, ngc) = self.archive.into_archives();

        let mut name_file = File::create(&self.paths.n)?;
        let mut data_file = File::create(&self.paths.d)?;
        dgc.write_to(&mut data_file)?;
        ngc.write_to(&mut name_file)?;

        self.set_need_save(false);

        Ok(())
    }

    /// Save the archive as another file
    pub fn save_as(&mut self, new_path: ArchivePathPair) -> CResult<()> {
        let prev_path = self.paths.clone();
        self.paths = new_path;
        let result = self.save();
        // If there's an error, revert to previous path
        if let Err(_) = &result {
            self.paths = prev_path;
        }
        result
    }
}
