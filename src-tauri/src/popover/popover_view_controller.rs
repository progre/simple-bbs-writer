use std::cell::{OnceCell, RefCell};

use crate::popover::popover_view_model::PopoverViewModel;

use super::popover_view::create_popover_view;
use dispatch2::{run_on_main, MainThreadBound};
use objc2::{define_class, msg_send, rc::Retained, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSButton, NSControlTextEditingDelegate, NSTextField, NSTextFieldDelegate,
    NSTextView, NSViewController,
};
use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol, NSString};

pub struct PopoverViewControllerIvars {
    view_model: RefCell<PopoverViewModel>,
    text_view: OnceCell<Retained<NSTextView>>,
    sage_checkbox: OnceCell<Retained<NSButton>>,
}

impl Default for PopoverViewControllerIvars {
    fn default() -> Self {
        Self {
            view_model: RefCell::new(PopoverViewModel::new()),
            text_view: OnceCell::new(),
            sage_checkbox: OnceCell::new(),
        }
    }
}

define_class!(
    #[unsafe(super(NSViewController))]
    #[thread_kind = MainThreadOnly]
    #[ivars = PopoverViewControllerIvars]
    pub struct PopoverViewController;

    unsafe impl NSObjectProtocol for PopoverViewController {}

    unsafe impl NSControlTextEditingDelegate for PopoverViewController {
        #[unsafe(method(controlTextDidChange:))]
        fn control_text_did_change(&self, notification: &NSNotification) {
            self.url_text_did_change(notification);
        }
    }

    unsafe impl NSTextFieldDelegate for PopoverViewController {}

    impl PopoverViewController {
        #[unsafe(method(loadView))]
        fn load_view(&self) {
            let mtm = MainThreadMarker::new().unwrap();
            self.load_view_impl(mtm);
        }

        #[unsafe(method(postButtonDidClick:))]
        fn post_button_did_click(&self, _sender: &NSObject) {
            self.post_button_did_click_impl();
        }

        #[unsafe(method(sageCheckboxDidChange:))]
        fn sage_checkbox_did_change(&self, _sender: &NSButton) {
            self.sage_checkbox_did_change_impl();
        }

        #[unsafe(method(commentTextViewDidChange:))]
        fn comment_text_view_did_change(&self, notification: &NSNotification) {
            self.comment_text_view_did_change_impl(notification);
        }

        #[unsafe(method(closeButtonDidClick:))]
        fn close_button_did_click(&self, _sender: &NSObject) {
            let mtm = MainThreadMarker::new().unwrap();
            self.close_button_did_click_impl(mtm);
        }
    }
);

impl PopoverViewController {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PopoverViewControllerIvars::default());
        unsafe { msg_send![super(this), init] }
    }

    fn load_view_impl(&self, mtm: MainThreadMarker) {
        let (view, text_view, sage_checkbox) = create_popover_view(mtm, self);

        self.ivars().text_view.set(text_view).unwrap();
        self.ivars()
            .sage_checkbox
            .set(sage_checkbox.clone())
            .unwrap();
        self.setView(&view);

        // ViewModelの初期値をビューに反映
        let initial_sage = self.ivars().view_model.borrow().get_sage();
        sage_checkbox.setState(if initial_sage { 1 } else { 0 });

        self.subscribe_to_comment_changes(mtm);
    }

    fn subscribe_to_comment_changes(&self, mtm: MainThreadMarker) {
        let text_view = self.ivars().text_view.get().unwrap().clone();
        let mtb = MainThreadBound::new(text_view, mtm);

        self.ivars()
            .view_model
            .borrow_mut()
            .subscribe_comment(move |comment| {
                run_on_main(|mtm| {
                    let text_view = mtb.get(mtm);
                    let ns_string = NSString::from_str(&comment);
                    if ns_string != text_view.string() {
                        text_view.setString(&ns_string);
                    }
                });
            });
    }
    fn post_button_did_click_impl(&self) {
        self.ivars().view_model.borrow_mut().on_post_clicked();
    }

    fn sage_checkbox_did_change_impl(&self) {
        let checkbox = self.ivars().sage_checkbox.get().unwrap();
        let state = checkbox.state();
        let sage = state == 1; // NSControlStateValue::On
        self.ivars().view_model.borrow_mut().set_sage(sage);
    }

    fn url_text_did_change(&self, notification: &NSNotification) {
        let object = notification.object().unwrap();
        let text_field = object.downcast::<NSTextField>().unwrap();
        let text = text_field.stringValue();
        let text_str = text.to_string();
        self.ivars().view_model.borrow_mut().set_url(text_str);
    }

    fn comment_text_view_did_change_impl(&self, notification: &NSNotification) {
        let object = notification.object().unwrap();
        let text_view = object.downcast::<NSTextView>().unwrap();
        let text = text_view.string();
        let text_str = text.to_string();
        self.ivars().view_model.borrow_mut().set_comment(text_str);
    }

    fn close_button_did_click_impl(&self, mtm: MainThreadMarker) {
        let app = NSApplication::sharedApplication(mtm);
        app.terminate(None);
    }
}
