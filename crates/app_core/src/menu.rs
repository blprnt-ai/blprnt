use common::paths::BlprntPath;
use tauri::AppHandle;
use tauri::Result;
use tauri::Wry;
use tauri::image::Image;
use tauri::menu::AboutMetadataBuilder;
use tauri::menu::Menu;
use tauri::menu::MenuItem;
use tauri::menu::PredefinedMenuItem;
use tauri::menu::SubmenuBuilder;

pub const REPORT_BUG_MENU_ITEM_ID: &str = "report_bug";
pub const DOCUMENTATION_MENU_ITEM_ID: &str = "documentation";
pub const VIEW_LICENSE_MENU_ITEM_ID: &str = "view_license";

pub fn create_menu(app: &AppHandle, report_bug_enabled: bool) -> Result<Menu<Wry>> {
  let report_bug_item =
    MenuItem::with_id(app, REPORT_BUG_MENU_ITEM_ID, "Report Bug", report_bug_enabled, None::<&str>)?;

  let documentation_item = MenuItem::with_id(app, DOCUMENTATION_MENU_ITEM_ID, "Documentation", true, None::<&str>)?;
  let view_license_item = MenuItem::with_id(app, VIEW_LICENSE_MENU_ITEM_ID, "View License", true, None::<&str>)?;

  let image_path = BlprntPath::app_resources().join("brand/logo.png");
  let image = Image::from_path(image_path).ok();

  let mut about_metadata =
    AboutMetadataBuilder::new().website(Some("https://blprnt.ai".to_string())).name(Some("blprnt".to_string()));
  if let Some(image) = image {
    about_metadata = about_metadata.icon(Some(image));
  }
  let about_metadata = about_metadata.build();

  let menu = Menu::with_items(
    app,
    &[
      &SubmenuBuilder::new(app, "blprnt")
        .items(&[
          &PredefinedMenuItem::about(app, None, Some(about_metadata))?,
          &PredefinedMenuItem::separator(app)?,
          &PredefinedMenuItem::services(app, None)?,
          &PredefinedMenuItem::separator(app)?,
          &PredefinedMenuItem::hide(app, None)?,
          &PredefinedMenuItem::quit(app, None)?,
        ])
        .build()?,
      &SubmenuBuilder::new(app, "Edit")
        .items(&[
          &PredefinedMenuItem::undo(app, None)?,
          &PredefinedMenuItem::redo(app, None)?,
          &PredefinedMenuItem::separator(app)?,
          &PredefinedMenuItem::cut(app, None)?,
          &PredefinedMenuItem::copy(app, None)?,
          &PredefinedMenuItem::paste(app, None)?,
          &PredefinedMenuItem::select_all(app, None)?,
        ])
        .build()?,
      &SubmenuBuilder::new(app, "Window")
        .items(&[
          &PredefinedMenuItem::minimize(app, None)?,
          &PredefinedMenuItem::maximize(app, None)?,
          #[cfg(not(target_os = "windows"))]
          &PredefinedMenuItem::fullscreen(app, None)?,
          &PredefinedMenuItem::separator(app)?,
        ])
        .build()?,
      &SubmenuBuilder::new(app, "Help")
        .items(&[&documentation_item, &report_bug_item, &PredefinedMenuItem::separator(app)?, &view_license_item])
        .build()?,
    ],
  )?;

  Ok(menu)
}

pub fn none(app: &AppHandle) -> Result<Menu<Wry>> {
  let menu = Menu::with_items(app, &[])?;

  Ok(menu)
}
