use gtk::{self, Button, Window, WindowType, HeaderBar, Notebook, FileChooserAction};
use gtk::prelude::*;
use super::page::Page;
use util;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeSet;

pub struct Application {
    pub window: Window,
    pub pages: Vec<Rc<RefCell<Page>>>,
    pub notebook: Notebook,
}

impl Application {
    pub fn new() -> Rc<RefCell<Application>> {
        let window = Window::new(WindowType::Toplevel);
        window.set_title("Chum World");
        window.set_default_size(640, 480);
        window.set_wmclass("chum-world", "Chum World");

        let header = HeaderBar::new();
        header.set_title("Chum World");
        header.set_show_close_button(true);
        window.set_titlebar(&header);
        
        let btn_open = Button::new_with_label("Open");
        let btn_save = Button::new_with_label("Save");

        header.pack_start(&btn_open);
        header.pack_end(&btn_save);

        let notebook = Notebook::new();
        notebook.set_scrollable(true);
        window.add(&notebook);
        
        let evdel_window = window.clone();
        window.connect_delete_event(move |_, _| {
            if util::ask_confirmation(&evdel_window, "Are you sure you want to quit?") {
                gtk::main_quit();
                Inhibit(false)
            } else {
                Inhibit(true)
            }
        });

        let app = Rc::new(RefCell::new(Application {
            window: window,
            pages: Vec::new(),
            notebook: notebook,
        }));

        let btn_open_app = Rc::downgrade(&app);
        btn_open.connect_clicked(move |_| {
            let btn_open_app = btn_open_app.upgrade().unwrap();
            let value = util::open_gc(Path::new("/"), &btn_open_app.borrow().window, FileChooserAction::Open);
            if let Some(paths) = value {
                let page = Page::new(&btn_open_app, paths).unwrap();
                Application::add_page(&btn_open_app, &page);
            }
        });

        app
    }

    pub fn add_page(app: &Rc<RefCell<Application>>, page: &Rc<RefCell<Page>>) {
        let label = page.borrow().label.clone();
        let btn_close = gtk::Button::new_from_icon_name(*gtk::STOCK_CLOSE, gtk::IconSize::Menu.into());
        btn_close.set_relief(gtk::ReliefStyle::None);
        btn_close.set_tooltip_text("Close tab");
        btn_close.set_focus_on_click(false);
        let tab = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        tab.add(&label);
        tab.add(&btn_close);
        let container = page.borrow().container.clone();
        let id = app.borrow().notebook.append_page(&container, Some(&tab));
        app.borrow().notebook.set_tab_reorderable(&container, true);
        container.show_all();
        tab.show_all();
        app.borrow_mut().pages.push(page.clone());
        println!("{}", id);
        let btn_close_win = app.borrow().window.clone();
        let weakpg = Rc::downgrade(page);
        let weakap = Rc::downgrade(app);
        btn_close.connect_clicked(move |_| {
            if util::ask_confirmation(&btn_close_win, "Are you sure you want to close this tab?") {
                container.destroy();
                tab.destroy();
                let page = weakpg.upgrade().unwrap();
                let app = weakap.upgrade().unwrap();
                let mut app = app.borrow_mut();
                for i in 0..app.pages.len() {
                    if Rc::ptr_eq(&app.pages[i], &page) {
                        app.pages.remove(i);
                        break;
                    }
                }
                //self.pages.remove(&weakpg.upgrade().unwrap());
            }
        });

    }
}

