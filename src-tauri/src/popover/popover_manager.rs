use super::popover_view_controller::PopoverViewController;
use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::{NSPopover, NSPopoverBehavior, NSView};
use objc2_foundation::{NSRect, NSRectEdge};

pub struct PopoverManager {
    popover: Retained<NSPopover>,
}

impl PopoverManager {
    pub fn new(mtm: MainThreadMarker, view_controller: Retained<PopoverViewController>) -> Self {
        let popover = NSPopover::new(mtm);
        popover.setContentViewController(Some(&view_controller));
        // フォアグラウンドから外れたときに自動で閉じるように設定
        popover.setBehavior(NSPopoverBehavior::Transient);

        PopoverManager { popover }
    }

    pub fn is_shown(&self) -> bool {
        self.popover.isShown()
    }

    pub fn close(&self) {
        unsafe { self.popover.performClose(None) };
    }

    pub fn show(&self, view: &NSView, rect: NSRect) {
        self.popover.showRelativeToRect_ofView_preferredEdge(
            rect,
            view,
            NSRectEdge(1), // NSRectEdgeMinY
        );
    }
}
