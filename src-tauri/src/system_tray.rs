use crate::popover::{PopoverManager, PopoverViewController};
use objc2::rc::Retained;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSStatusBar, NSStatusItem};
use objc2_foundation::{NSObject, NSString};

pub struct StatusItemDelegateIvars {
    status_item: Retained<NSStatusItem>,
    popover_manager: PopoverManager,
}

impl StatusItemDelegateIvars {
    pub fn new(status_item: Retained<NSStatusItem>, popover_manager: PopoverManager) -> Self {
        StatusItemDelegateIvars {
            status_item,
            popover_manager,
        }
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = StatusItemDelegateIvars]
    pub struct StatusItemDelegate;

    impl StatusItemDelegate {
        /// システムトレイアイコンがクリックされた時に呼ばれるメソッド
        #[unsafe(method(statusItemDidClicked:))]
        fn status_item_did_clicked(&self, _sender: &NSObject) {
            let mtm = MainThreadMarker::new().unwrap();
            self.status_item_did_clicked_impl(mtm);
        }
    }
);

impl StatusItemDelegate {
    fn new(
        mtm: MainThreadMarker,
        status_item: Retained<NSStatusItem>,
        popover_manager: PopoverManager,
    ) -> Retained<Self> {
        let ivars = StatusItemDelegateIvars::new(status_item, popover_manager);
        let this = Self::alloc(mtm).set_ivars(ivars);
        unsafe { msg_send![super(this), init] }
    }

    fn status_item_did_clicked_impl(&self, mtm: MainThreadMarker) {
        let button = self.ivars().status_item.button(mtm).unwrap();
        let popover = &self.ivars().popover_manager;
        if popover.is_shown() {
            popover.close();
        } else {
            popover.show(&button, button.bounds());
        }
    }
}

pub struct SystemTray {
    _status_item: Retained<NSStatusItem>,
    _delegate: Retained<StatusItemDelegate>,
}

impl SystemTray {
    pub fn new(mtm: MainThreadMarker, view_controller: Retained<PopoverViewController>) -> Self {
        // NSStatusBarからステータスアイテムを作成
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(-1.0); // NSVariableStatusItemLength

        // ステータスアイテムのタイトルを設定
        let title = NSString::from_str("📝");
        let button = status_item.button(mtm).unwrap();
        button.setTitle(&title);

        let popover_manager = PopoverManager::new(mtm, view_controller);
        let delegate = StatusItemDelegate::new(mtm, status_item.clone(), popover_manager);

        // ステータスアイテムにアクションとターゲットを設定
        unsafe { button.setTarget(Some(&*delegate)) };
        unsafe { button.setAction(Some(objc2::sel!(statusItemDidClicked:))) };

        SystemTray {
            _status_item: status_item,
            _delegate: delegate,
        }
    }
}
