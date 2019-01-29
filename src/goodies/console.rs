use stdweb::{_js_impl, js};
use stdweb::{traits::*, web::document};

pub fn message(msg: &str) {
    let overlay = document().get_element_by_id("overlay").unwrap();
    let loading = document().get_element_by_id("loading").unwrap();

    js!(@{&loading}.textContent += " " + @{msg});

    js!(@{&overlay}.style.display = "block");
}

pub fn log(msg: &str) {
    js!(console.log(@{msg}));
}
