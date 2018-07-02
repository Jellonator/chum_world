use gtk;
use gtk::prelude::*;

pub mod app;
pub mod page;
pub mod editor;

pub fn begin() -> super::CResult<()> {
    gtk::init()?;

    let app = app::Application::new();
    app.borrow().window.show_all();

    gtk::main();

    Ok(())
}
