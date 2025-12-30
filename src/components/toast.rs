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
    let mut is_fading = use_signal(|| false);

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

    // Start fade out animation after 2.7 seconds (300ms before removal)
    use_effect(move || {
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(2700)).await;
            is_fading.set(true);
        });
    });

    let remove_toast = move |_| {
        is_fading.set(true);
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            let mut current_toasts = toasts.write();
            *current_toasts = current_toasts
                .iter()
                .filter(|t| t.id != toast_id)
                .cloned()
                .collect();
        });
    };

    let final_class = if is_fading() {
        format!("{} fade-out", class_name)
    } else {
        class_name.to_string()
    };

    rsx! {
        div { class: "{final_class}",
            span { class: "toast-icon", "{icon}" }
            span { class: "toast-message", "{toast.message}" }
            button {
                class: "toast-close",
                onclick: remove_toast,
                Icon {
                    icon: FaXmark,
                    width: 20,
                    height: 20,
                }
            }
            div { class: "toast-progress" }
        }
    }
}