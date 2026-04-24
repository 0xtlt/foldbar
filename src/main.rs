use std::cell::{Cell, RefCell};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSControlStateValueOff, NSControlStateValueOn,
    NSMenu, NSMenuItem, NSScreen, NSStatusBar, NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, ns_string};
use objc2_service_management::{SMAppService, SMAppServiceStatus};

const EXPANDED_SEPARATOR_LENGTH: f64 = 20.0;
const MIN_COLLAPSED_SEPARATOR_LENGTH: f64 = 500.0;
const MAX_COLLAPSED_SEPARATOR_LENGTH: f64 = 4000.0;
const DEFAULT_SCREEN_WIDTH: f64 = 1728.0;

#[derive(Default)]
struct ControllerIvars {
    toggle_item: RefCell<Option<Retained<NSStatusItem>>>,
    separator_item: RefCell<Option<Retained<NSStatusItem>>>,
    launch_at_login_item: RefCell<Option<Retained<NSMenuItem>>>,
    collapsed: Cell<bool>,
    toggling: Cell<bool>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ControllerIvars]
    struct FoldbarController;

    impl FoldbarController {
        #[unsafe(method(toggle:))]
        fn toggle(&self, _sender: Option<&AnyObject>) {
            if self.ivars().toggling.replace(true) {
                return;
            }

            if self.ivars().collapsed.get() {
                self.expand();
            } else {
                self.collapse();
            }

            self.ivars().toggling.set(false);
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _sender: Option<&AnyObject>) {
            if let Some(mtm) = MainThreadMarker::new() {
                NSApplication::sharedApplication(mtm).terminate(None);
            }
        }

        #[unsafe(method(toggleLaunchAtLogin:))]
        fn toggle_launch_at_login(&self, _sender: Option<&AnyObject>) {
            let service = unsafe { SMAppService::mainAppService() };
            let enabled = unsafe { service.status() } == SMAppServiceStatus::Enabled;
            let result = unsafe {
                if enabled {
                    service.unregisterAndReturnError()
                } else {
                    service.registerAndReturnError()
                }
            };

            if let Err(error) = result {
                eprintln!(
                    "Unable to update launch at login: {}",
                    error.localizedDescription()
                );

                if unsafe { service.status() } == SMAppServiceStatus::RequiresApproval {
                    unsafe { SMAppService::openSystemSettingsLoginItems() };
                }
            }

            self.update_launch_at_login_item();
        }
    }
);

impl FoldbarController {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ControllerIvars::default());
        unsafe { msg_send![super(this), init] }
    }

    fn setup(&self, mtm: MainThreadMarker) {
        let status_bar = NSStatusBar::systemStatusBar();

        let toggle_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);
        // Stable names let AppKit preserve menu bar ordering after Command-drag.
        toggle_item.setAutosaveName(Some(ns_string!("foldbar.toggle")));
        if let Some(button) = toggle_item.button(mtm) {
            button.setTitle(ns_string!("‹"));
            unsafe {
                button.setTarget(Some(self));
                button.setAction(Some(sel!(toggle:)));
            }
        }

        let separator_item = status_bar.statusItemWithLength(EXPANDED_SEPARATOR_LENGTH);
        if let Some(button) = separator_item.button(mtm) {
            button.setTitle(ns_string!("|"));
        }
        // The separator is the movable boundary between visible and hidden icons.
        separator_item.setAutosaveName(Some(ns_string!("foldbar.separator")));
        separator_item.setMenu(Some(&self.make_menu(mtm)));

        self.ivars().toggle_item.replace(Some(toggle_item));
        self.ivars().separator_item.replace(Some(separator_item));
    }

    fn make_menu(&self, mtm: MainThreadMarker) -> Retained<NSMenu> {
        let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!("Foldbar"));
        let launch_at_login_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                ns_string!("Launch at Login"),
                Some(sel!(toggleLaunchAtLogin:)),
                ns_string!(""),
            )
        };
        let quit_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                ns_string!("Quit Foldbar"),
                Some(sel!(quit:)),
                ns_string!("q"),
            )
        };

        unsafe {
            launch_at_login_item.setTarget(Some(self));
            quit_item.setTarget(Some(self));
        }

        self.ivars()
            .launch_at_login_item
            .replace(Some(launch_at_login_item.clone()));
        self.update_launch_at_login_item();

        menu.addItem(&launch_at_login_item);
        menu.addItem(&quit_item);
        menu
    }

    fn update_launch_at_login_item(&self) {
        let Some(item) = self.ivars().launch_at_login_item.borrow().as_ref().cloned() else {
            return;
        };

        let status = unsafe { SMAppService::mainAppService().status() };
        let state = if status == SMAppServiceStatus::Enabled {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        };

        item.setState(state);
    }

    fn collapse(&self) {
        let length = collapsed_separator_length();
        if let Some(separator_item) = self.ivars().separator_item.borrow().as_ref() {
            // A very wide separator pushes intervening menu bar items offscreen,
            // matching Hidden Bar's core technique without private APIs.
            separator_item.setLength(length);
        }
        if let Some(toggle_item) = self.ivars().toggle_item.borrow().as_ref() {
            if let Some(mtm) = MainThreadMarker::new() {
                if let Some(button) = toggle_item.button(mtm) {
                    button.setTitle(ns_string!("›"));
                }
            }
        }
        self.ivars().collapsed.set(true);
    }

    fn expand(&self) {
        if let Some(separator_item) = self.ivars().separator_item.borrow().as_ref() {
            separator_item.setLength(EXPANDED_SEPARATOR_LENGTH);
        }
        if let Some(toggle_item) = self.ivars().toggle_item.borrow().as_ref() {
            if let Some(mtm) = MainThreadMarker::new() {
                if let Some(button) = toggle_item.button(mtm) {
                    button.setTitle(ns_string!("‹"));
                }
            }
        }
        self.ivars().collapsed.set(false);
    }
}

fn collapsed_separator_length() -> f64 {
    let screen_width = MainThreadMarker::new()
        .and_then(NSScreen::mainScreen)
        .map(|screen| screen.visibleFrame().size.width)
        .unwrap_or(DEFAULT_SCREEN_WIDTH);

    (screen_width + 200.0).clamp(
        MIN_COLLAPSED_SEPARATOR_LENGTH,
        MAX_COLLAPSED_SEPARATOR_LENGTH,
    )
}

fn main() {
    let mtm = MainThreadMarker::new().expect("Foldbar must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    // Accessory keeps Foldbar out of the Dock while still allowing status items.
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let controller = FoldbarController::new(mtm);
    controller.setup(mtm);

    app.finishLaunching();
    app.run();
}
