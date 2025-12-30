use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaXmark;
use dioxus_free_icons::Icon;

// Toast notification type
#[derive(Clone, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub id: usize,
}

#[component]
pub fn ToastContainer(toasts: Signal<Vec<Toast>>) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/compiled/toast.css") }
        div { class: "toast-container",
            for toast in toasts().iter() {
                ToastItem { key: "{toast.id}", toast: toast.clone(), toasts: toasts }
            }
        }
    }
}

#[component]
fn ToastItem(toast: Toast, toasts: Signal<Vec<Toast>>) -> Element {
    let toast_id = toast.id;

    let class_name = match toast.toast_type {
        ToastType::Success => "toast toast-success",
        ToastType::Error => "toast toast-error",
        ToastType::Info => "toast toast-info",
    };

    let icon = match toast.toast_type {
        ToastType::Success => "✓",
        ToastType::Error => "✕",
        ToastType::Info => "ℹ",
    };

    let remove_toast = move |_| {
        let mut current_toasts = toasts.write();
        *current_toasts = current_toasts
            .iter()
            .filter(|t| t.id != toast_id)
            .cloned()
            .collect();
    };

    rsx! {
        div { class: "{class_name}",
            span { class: "toast-icon", "{icon}" }
            span { class: "toast-message", "{toast.message}" }
            button { class: "toast-close", onclick: remove_toast, Icon {
                        icon: FaXmark,
                        width: 20,
                        height: 20,
                    } }
        }
    }
}