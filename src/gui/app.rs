use gtk::{self, MenuItemExt, Button, HeaderBar, Notebook, FileChooserAction};
use gtk::prelude::*;
use super::page::Page;
use util;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use ::CResult;

/// Represents an application
/// The pages property maps page tab ids to Page objects
pub struct Application {
    pub window: gtk::Window,
    pub pages: Vec<Rc<RefCell<Page>>>,
    pub notebook: Notebook,
    pub archive_buttons: Vec<gtk::Widget>,
    pub selected: usize,
}

pub fn action_open_file(app: &Rc<RefCell<Application>>) -> CResult<()> {
    let path = env::current_dir()?;
    let value = util::open_gc(&path, &app.borrow().window, FileChooserAction::Open);
    if let Some(paths) = value {
        for page in &app.borrow().pages {
            if page.borrow().paths == paths {
                return Ok(());
            }
        }
        let page = Page::new(&app, paths)?;
        Application::add_page(&app, &page);
    }
    Ok(())
}

pub fn action_save_file(app: &Rc<RefCell<Application>>) -> CResult<()> {
    let current_page = app.borrow().get_current_page().unwrap().clone();
    current_page.borrow_mut().save()?;
    Ok(())
}

pub fn action_saveas(app: &Rc<RefCell<Application>>) -> CResult<()> {
    let current_page = app.borrow().get_current_page().unwrap().clone();
    let path = current_page.borrow().paths.d.parent().unwrap().to_owned();
    let value = util::open_gc(&path, &app.borrow().window, FileChooserAction::Save);
    if let Some(paths) = value {
        current_page.borrow_mut().save_as(paths)?;
    }
    Ok(())
}

impl Application {
    /// Get the current page ID
    pub fn get_current_page_id(&self) -> Option<u32> {
        self.notebook.get_current_page()
    }

    /// Get the currently opened page
    pub fn get_current_page(&self) -> Option<&Rc<RefCell<Page>>> {
        self.notebook.get_current_page()
                     .and_then(|id| self.pages.get(id as usize))
    }

    /// Create a new application window
    pub fn new() -> Rc<RefCell<Application>> {
        // create window
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("Chum World");
        window.set_default_size(640, 480);
        window.set_wmclass("chum-world", "Chum World");
        // Create the header bar
        let header = HeaderBar::new();
        header.set_title("Chum World");
        header.set_show_close_button(true);
        window.set_titlebar(&header);
        // Add buttons to the header bar
        let btn_open = Button::new_with_label("Open");
        let btn_save = Button::new_with_label("Save");
        let btn_menu = gtk::MenuButton::new();
        header.pack_start(&btn_open);
        header.pack_end(&btn_menu);
        header.pack_end(&btn_save);
        // Create the menu
        let menu = gtk::Menu::new();
        let item_saveas = gtk::MenuItem::new_with_label("Save As");
        //let item_extract = gtk::MenuItem::new_with_label("Extract Json");
        //let item_import = gtk::MenuItem::new_with_label("Import Json");
        menu.append(&item_saveas);
        //menu.append(&item_extract);
        //menu.append(&item_import);
        menu.show_all();
        btn_menu.set_popup(Some(&menu));
        // Add notebook tabs to the application
        let notebook = Notebook::new();
        notebook.set_scrollable(true);
        window.add(&notebook);
        // Confirmation when the user closes the window
        let evdel_window = window.clone();
        window.connect_delete_event(move |_, _| {
            if util::ask_confirmation(&evdel_window, "Are you sure you want to quit?") {
                gtk::main_quit();
                Inhibit(false)
            } else {
                Inhibit(true)
            }
        });
        // create app
        let app = Rc::new(RefCell::new(Application {
            window: window,
            pages: Vec::new(),
            notebook: notebook.clone(),
            archive_buttons: vec![btn_save.clone().upcast(), btn_menu.clone().upcast()],
            selected: 0,
        }));
        // handle open button
        let btn_open_app = Rc::downgrade(&app);
        btn_open.connect_clicked(move |_| {
            let app = btn_open_app.upgrade().unwrap();
            util::handle_result(action_open_file(&app), "Error opening file", &app.borrow().window);
        });
        // handle save button
        let btn_save_app = Rc::downgrade(&app);
        btn_save.connect_clicked(move |_| {
            let app = btn_save_app.upgrade().unwrap();
            util::handle_result(action_save_file(&app), "Error saving file", &app.borrow().window);
        });
        // These callbacks are needed in order to keep pages consistend with
        // the notebook tab order.
        let rapp = Rc::downgrade(&app);
        notebook.connect_page_removed(move |_, _, id| {
            let app = rapp.upgrade().unwrap();
            app.borrow_mut().pages.remove(id as usize);
            app.borrow().update_save_button();
        });
        let oapp = Rc::downgrade(&app);
        notebook.connect_page_reordered(move |_, _, id| {
            let app = oapp.upgrade().unwrap();
            let selected = app.borrow().selected as usize;
            let value = app.borrow_mut().pages.remove(selected as usize);
            app.borrow_mut().pages.insert(id as usize, value);
            app.borrow_mut().selected = id as usize;
        });
        let sapp = Rc::downgrade(&app);
        notebook.connect_switch_page(move |_, _, id| {
            let app = sapp.upgrade().unwrap();
            app.borrow_mut().selected = id as usize;
        });
        // Create menu actions
        let btn_saveas_app = Rc::downgrade(&app);
        item_saveas.connect_activate(move |_| {
            let app = btn_saveas_app.upgrade().unwrap();
            util::handle_result(action_saveas(&app), "Error saving file", &app.borrow().window);
        });
        // Update save button
        app.borrow().update_save_button();
        app
    }

    // Disables the save button if there are no open files
    pub fn update_save_button(&self) {
        let value = if let Some(_value) = self.notebook.get_current_page() {
            true
        } else {
            false
        };

        for btn in &self.archive_buttons {
            btn.set_sensitive(value);
        }
    }

    // Add a new page to this Application
    pub fn add_page(app: &Rc<RefCell<Application>>, page: &Rc<RefCell<Page>>) {
        // Create the tab. The tab has a label and a close button.
        let label = page.borrow().label.clone();
        let btn_close = gtk::Button::new_from_icon_name(*gtk::STOCK_CLOSE, gtk::IconSize::Menu.into());
        btn_close.set_relief(gtk::ReliefStyle::None);
        btn_close.set_tooltip_text("Close tab");
        btn_close.set_focus_on_click(false);
        let tab = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        tab.add(&label);
        tab.add(&btn_close);
        // Add the container to the notebook with the above tab.
        let container = page.borrow().container.clone();
        let notebook = app.borrow().notebook.clone();
        let id = notebook.append_page(&container, Some(&tab));
        notebook.set_tab_reorderable(&container, true);
        container.show_all();
        tab.show_all();
        app.borrow_mut().pages.insert(id as  usize, page.clone());
        notebook.set_current_page(Some(id));
        // Create the event for when the tab is closed.
        let btn_close_win = app.borrow().window.clone();
        let weakpg = Rc::downgrade(page);
        let weakap = Rc::downgrade(app);
        btn_close.connect_clicked(move |_| {
            if util::ask_confirmation(&btn_close_win, "Are you sure you want to close this tab?") {
                let page = weakpg.upgrade().unwrap();
                page.borrow().container.destroy();
                tab.destroy();
                // app.pages will be automatically removed
            }
        });
        app.borrow().update_save_button();
    }
}
