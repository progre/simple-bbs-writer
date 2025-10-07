use objc2::{rc::Retained, runtime::ProtocolObject, sel, MainThreadMarker};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBorderType, NSButton, NSScrollView, NSTextField, NSTextView,
    NSView,
};
use objc2_foundation::{NSNotificationCenter, NSPoint, NSRect, NSSize, NSString};

use crate::popover::popover_view_controller::PopoverViewController;

fn create_url_text_field(
    mtm: MainThreadMarker,
    target: &PopoverViewController,
) -> Retained<NSTextField> {
    let url_field = NSTextField::new(mtm);
    url_field.setTranslatesAutoresizingMaskIntoConstraints(false);
    let placeholder = NSString::from_str("掲示板のURLを入力");
    url_field.setPlaceholderString(Some(&placeholder));
    let delegate = ProtocolObject::from_ref(target);
    unsafe { url_field.setDelegate(Some(delegate)) };
    url_field
}

fn create_comment_inner_text_view(
    mtm: MainThreadMarker,
    target: &PopoverViewController,
) -> Retained<NSTextView> {
    let comment_text_view = NSTextView::new(mtm);
    comment_text_view.setAllowsUndo(true);
    comment_text_view.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);

    let notification_center = NSNotificationCenter::defaultCenter();
    unsafe {
        notification_center.addObserver_selector_name_object(
            target,
            sel!(commentTextViewDidChange:),
            Some(&NSString::from_str("NSTextDidChangeNotification")),
            Some(&*comment_text_view),
        )
    };

    comment_text_view
}

fn create_comment_text_view(
    mtm: MainThreadMarker,
    target: &PopoverViewController,
) -> (Retained<NSScrollView>, Retained<NSTextView>) {
    let scroll_view = NSScrollView::new(mtm);
    scroll_view.setTranslatesAutoresizingMaskIntoConstraints(false);
    scroll_view.setHasVerticalScroller(true);
    scroll_view.setBorderType(NSBorderType::BezelBorder);

    let comment_text_view = create_comment_inner_text_view(mtm, target);

    scroll_view.setDocumentView(Some(&*comment_text_view));

    (scroll_view, comment_text_view)
}

fn create_sage_checkbox(
    mtm: MainThreadMarker,
    target: &PopoverViewController,
) -> Retained<NSButton> {
    let sage_checkbox = NSButton::new(mtm);
    sage_checkbox.setTranslatesAutoresizingMaskIntoConstraints(false);
    sage_checkbox.setTitle(&NSString::from_str("sageで書き込む"));
    sage_checkbox.setButtonType(objc2_app_kit::NSButtonType::Switch);
    unsafe { sage_checkbox.setTarget(Some(target)) };
    unsafe { sage_checkbox.setAction(Some(sel!(sageCheckboxDidChange:))) };
    sage_checkbox
}

fn create_submit_button(
    mtm: MainThreadMarker,
    target: &PopoverViewController,
) -> Retained<NSButton> {
    let submit_button = NSButton::new(mtm);
    submit_button.setTranslatesAutoresizingMaskIntoConstraints(false);
    submit_button.setTitle(&NSString::from_str("書き込み"));
    submit_button.setButtonType(objc2_app_kit::NSButtonType::MomentaryPushIn);
    unsafe { submit_button.setTarget(Some(target)) };
    unsafe { submit_button.setAction(Some(sel!(postButtonDidClick:))) };
    submit_button
}

fn anchor(
    view: &Retained<NSView>,
    url_field: Retained<NSTextField>,
    scroll_view: Retained<NSScrollView>,
    sage_checkbox: &Retained<NSButton>,
    submit_button: Retained<NSButton>,
) {
    // URL text field constraints
    url_field
        .topAnchor()
        .constraintEqualToAnchor_constant(&view.topAnchor(), 10.0)
        .setActive(true);
    url_field
        .leadingAnchor()
        .constraintEqualToAnchor_constant(&view.leadingAnchor(), 10.0)
        .setActive(true);
    url_field
        .trailingAnchor()
        .constraintEqualToAnchor_constant(&view.trailingAnchor(), -10.0)
        .setActive(true);
    url_field
        .heightAnchor()
        .constraintEqualToConstant(25.0)
        .setActive(true);

    // Comment scroll view constraints
    scroll_view
        .topAnchor()
        .constraintEqualToAnchor_constant(&url_field.bottomAnchor(), 10.0)
        .setActive(true);
    scroll_view
        .leadingAnchor()
        .constraintEqualToAnchor_constant(&view.leadingAnchor(), 10.0)
        .setActive(true);
    scroll_view
        .trailingAnchor()
        .constraintEqualToAnchor_constant(&view.trailingAnchor(), -10.0)
        .setActive(true);
    scroll_view
        .heightAnchor()
        .constraintEqualToConstant(100.0)
        .setActive(true);

    // Sage checkbox constraints
    sage_checkbox
        .topAnchor()
        .constraintEqualToAnchor_constant(&scroll_view.bottomAnchor(), 10.0)
        .setActive(true);
    sage_checkbox
        .leadingAnchor()
        .constraintEqualToAnchor_constant(&view.leadingAnchor(), 10.0)
        .setActive(true);
    sage_checkbox
        .bottomAnchor()
        .constraintEqualToAnchor_constant(&view.bottomAnchor(), -10.0)
        .setActive(true);

    // Submit button constraints
    submit_button
        .topAnchor()
        .constraintEqualToAnchor_constant(&scroll_view.bottomAnchor(), 10.0)
        .setActive(true);
    submit_button
        .trailingAnchor()
        .constraintEqualToAnchor_constant(&view.trailingAnchor(), -10.0)
        .setActive(true);
    submit_button
        .bottomAnchor()
        .constraintEqualToAnchor_constant(&view.bottomAnchor(), -10.0)
        .setActive(true);
    submit_button
        .widthAnchor()
        .constraintEqualToConstant(80.0)
        .setActive(true);
}

pub fn create_popover_view(
    mtm: MainThreadMarker,
    target: &PopoverViewController,
) -> (Retained<NSView>, Retained<NSTextView>, Retained<NSButton>) {
    let view = NSView::new(mtm);
    let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(300.0, 200.0));
    view.setFrame(frame);

    let url_field = create_url_text_field(mtm, target);
    let (scroll_view, comment_text_view) = create_comment_text_view(mtm, target);
    let sage_checkbox = create_sage_checkbox(mtm, target);
    let submit_button = create_submit_button(mtm, target);

    view.addSubview(&url_field);
    view.addSubview(&scroll_view);
    view.addSubview(&sage_checkbox);
    view.addSubview(&submit_button);

    anchor(&view, url_field, scroll_view, &sage_checkbox, submit_button);

    (view, comment_text_view, sage_checkbox)
}
