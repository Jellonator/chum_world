use gtk::{Widget, Label};
use gtk::prelude::*;
use gui::page::{Page, ArchiveFile};
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, Read, Write};
use ::CResult;
use std::collections::HashMap;

pub mod text;

/// A plugin that can be used to import, export, or edit files.
pub trait FilePlugin {
    /// Take data from a reader and transform it into the actual archive's format
    /// For example, WAV -> DSP or raw image -> PNG
    fn import_data(&self, input: &mut Read, output: &mut Write) -> CResult<()>;
    /// Take data from a reader and transform it into a user-editable format
    /// For example, DSP -> WAV or PNG -> raw image
    fn export_data(&self, input: &mut Read, output: &mut Write) -> CResult<()>;
    /// Create an editor gui for the given file
    fn create_editor(&self, parent: &Rc<RefCell<Page>>, file: &Rc<RefCell<ArchiveFile>>) -> CResult<Widget>;
    /// Get the plugin type string
    fn get_plugin_string(&self) -> &'static str;
}

/// Manages plugins. Maps file types to individual plugin handlers.
pub struct PluginManager {
    pub plugins: HashMap<String, Box<FilePlugin>>,
    pub ftypes: HashMap<String, String>
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> PluginManager {
        let mut ret = PluginManager {
            plugins: HashMap::new(),
            ftypes: HashMap::new()
        };
        let txt = ret.register_plugin(Box::new(text::FilePluginLengthText));
        ret.register_for_type(txt, "TXT");
        ret
    }

    /// Register a new plugin
    pub fn register_plugin(&mut self, plugin: Box<FilePlugin>) -> &'static str {
        let ret = plugin.get_plugin_string();
        self.plugins.insert(plugin.get_plugin_string().to_owned(), plugin);
        ret
    }

    /// Associate a file type to a plugin string
    pub fn register_for_type(&mut self, fstr: &str, typestr: &str) {
        self.ftypes.insert(typestr.to_owned(), fstr.to_owned());
    }

    /// Create an editor pane for the given file
    pub fn create_editor(&self, parent: &Rc<RefCell<Page>>, file: &Rc<RefCell<ArchiveFile>>) -> Widget {
        let typestr: &str = &file.borrow().typeid;
        let result = match self.ftypes.get(typestr) {
            Some(name) => {
                let plugin = self.plugins.get(name).unwrap();
                plugin.create_editor(parent, file)
            },
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

    /// Import a file
    pub fn import<'a>(&self, typestr: &str, input: &mut Read, output: &mut Write) -> CResult<()> {
        if let Some(plugin) = self.ftypes.get(typestr).and_then(|fstring| {
            self.plugins.get(fstring)
        }) {
            let mut data = Vec::new();
            plugin.import_data(input, &mut data)?;
            let mut read: &mut io::Read = &mut &data[..];
            io::copy(read, output)?;
        } else {
            io::copy(input, output)?;
        }
        Ok(())
    }

    /// Export a file
    pub fn export<'a>(&self, typestr: &str, input: &mut Read, output: &mut Write) -> CResult<()> {
        if let Some(plugin) = self.ftypes.get(typestr).and_then(|fstring| {
            self.plugins.get(fstring)
        }) {
            let mut data = Vec::new();
            plugin.export_data(input, &mut data)?;
            let mut read: &mut io::Read = &mut &data[..];
            io::copy(read, output)?;
        } else {
            io::copy(input, output)?;
        }
        Ok(())
    }
}
