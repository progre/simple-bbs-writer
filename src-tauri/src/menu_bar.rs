use objc2::{
    rc::Retained,
    runtime::{AnyObject, Sel},
    sel, MainThreadMarker,
};
use objc2_app_kit::{NSMenu, NSMenuItem};
use objc2_foundation::NSString;

fn create_menu_item(
    mtm: MainThreadMarker,
    title: &str,
    key_equivalent: &str,
    target: Option<&AnyObject>,
    action: Sel,
) -> Retained<NSMenuItem> {
    let item = NSMenuItem::new(mtm);
    item.setTitle(&NSString::from_str(title));
    item.setKeyEquivalent(&NSString::from_str(key_equivalent));
    if let Some(target) = target {
        unsafe { item.setTarget(Some(target)) };
    }
    unsafe { item.setAction(Some(action)) };
    item
}

fn create_edit_menu(
    mtm: MainThreadMarker,
    post_target: &AnyObject,
    post_sel: Sel,
) -> Retained<NSMenu> {
    let edit_menu = NSMenu::new(mtm);
    edit_menu.setTitle(&NSString::from_str("Edit"));

    edit_menu.addItem(&create_menu_item(mtm, "Undo", "z", None, sel!(undo:)));
    edit_menu.addItem(&create_menu_item(mtm, "Cut", "x", None, sel!(cut:)));
    edit_menu.addItem(&create_menu_item(mtm, "Copy", "c", None, sel!(copy:)));
    edit_menu.addItem(&create_menu_item(mtm, "Paste", "v", None, sel!(paste:)));
    let item = create_menu_item(mtm, "Select All", "a", None, sel!(selectAll:));
    edit_menu.addItem(&item);
    let item = create_menu_item(mtm, "Post", "\r", Some(post_target), post_sel);
    edit_menu.addItem(&item);

    edit_menu
}

pub fn create_menu_bar(
    mtm: MainThreadMarker,
    post_target: &AnyObject,
    post_sel: Sel,
) -> Retained<NSMenu> {
    let menu_bar = NSMenu::new(mtm);
    let edit_menu_item = NSMenuItem::new(mtm);
    let edit_menu = create_edit_menu(mtm, post_target, post_sel);
    edit_menu_item.setSubmenu(Some(&edit_menu));
    menu_bar.addItem(&edit_menu_item);
    menu_bar
}
