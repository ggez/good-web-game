use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, path};
use stdweb::{_js_impl, js};
use stdweb::{
    traits::*,
    web::{document, html_element::ImageElement, window, XmlHttpRequest},
};

use super::{File, Filesystem};
use crate::{conf, GameResult};

type Url<'a> = &'a str;

fn progress_string(progress: f64) -> String {
    let a = progress * 10.;
    let b = 10. - progress * 10. + 1.;
    format!(
        "[ {} ]",
        std::iter::repeat('|')
            .take(a as usize)
            .chain(std::iter::repeat('-').take(b as usize))
            .collect::<String>()
    )
}

fn show_loading() {
    let overlay = document().get_element_by_id("overlay").unwrap();
    let loading = document().get_element_by_id("loading").unwrap();

    js!(@{&loading}.textContent = @{progress_string(0.)});

    js!(@{&overlay}.style.display = "block");
}

fn hide_loading() {
    let overlay = document().get_element_by_id("overlay").unwrap();

    js!(@{&overlay}.style.display = "none");
}

fn update_progress(progress: f32) {
    let loading = document().get_element_by_id("loading").unwrap();
    js!(@{&loading}.textContent = @{progress_string(progress as f64)});
}

struct Cache {
    requests_amount: Option<usize>,
    files: HashMap<path::PathBuf, File>,
}
impl Cache {
    fn new() -> Rc<RefCell<Option<Cache>>> {
        let cache = Cache {
            requests_amount: None,
            files: HashMap::new(),
        };
        Rc::new(RefCell::new(Some(cache)))
    }

    fn is_loaded(&self) -> bool {
        self.requests_amount
            .map_or(false, |n| n == self.files.len())
    }

    fn progress(&self) -> f64 {
        if let Some(requests) = self.requests_amount {
            self.files.len() as f64 / requests as f64
        } else {
            0.
        }
    }
}

fn cache_file(cache: Rc<RefCell<Option<Cache>>>, path: path::PathBuf, file: File) {
    cache
        .borrow_mut()
        .as_mut()
        .unwrap()
        .files
        .insert(path, file);
}

fn load_text_file<F: 'static + Fn(&str)>(url: Url, f: F) {
    let request = XmlHttpRequest::new();
    let f = {
        let request = request.clone();
        move || f(&request.response_text().unwrap().unwrap())
    };

    js!(@{&request}.addEventListener("load", function () {@{f}()}));

    request.open("GET", &url[1..]).unwrap();
    request.send().unwrap();
}

fn load_image<F: 'static + Fn(ImageElement)>(url: Url, f: F) {
    let image_element = ImageElement::new();
    let f = {
        let image_element = image_element.clone();
        move || f(image_element.clone())
    };
    js!(@{&image_element}.addEventListener("load", function () {@{f}()}));

    image_element.set_src(&url[1..]);
}

fn animate<F: 'static + FnOnce(Filesystem) -> GameResult>(f: F, cache: Rc<RefCell<Option<Cache>>>) {
    {
        let progress = cache.borrow_mut().as_ref().unwrap().progress();
        update_progress(progress as f32);
    }
    {
        if cache.borrow_mut().as_ref().unwrap().is_loaded() {
            let cache = cache.borrow_mut().take().unwrap();
            let fs = Filesystem { files: cache.files };
            hide_loading();
            f(fs).unwrap();
            return;
        }
    }
    window().request_animation_frame(move |_| animate(f, cache));
}

fn load_cache(files: &[&str], cache: Rc<RefCell<Option<Cache>>>) {
    show_loading();

    {
        let mut cache = cache.borrow_mut();
        cache.as_mut().unwrap().requests_amount = Some(files.len());
    }

    for file in files.iter() {
        let path = path::Path::new(file).to_path_buf();
        let cache = cache.clone();

        match path.extension().and_then(std::ffi::OsStr::to_str).clone() {
            Some("png") => load_image(file, move |image| {
                cache_file(cache.clone(), path.clone(), File::Image(image))
            }),
            _ => load_text_file(file, move |text| {
                cache_file(
                    cache.clone(),
                    path.clone(),
                    File::Bytes(std::io::Cursor::new(text.as_bytes().to_vec())),
                )
            }),
        }
    }
}

pub fn mount<F>(cache_conf: conf::Cache, f: F)
where
    F: 'static + FnOnce(Filesystem) -> GameResult,
{
    let cache = Cache::new();

    show_loading();

    match cache_conf {
        conf::Cache::Index => {
            load_text_file("/index.txt", {
                let cache = cache.clone();
                move |index| {
                    let files = index.lines().collect::<Vec<_>>();
                    load_cache(&files, cache.clone());
                }
            });

            window().request_animation_frame(move |_| animate(f, cache));
        }
        conf::Cache::List(files) => {
            load_cache(&files, cache.clone());
            window().request_animation_frame(move |_| animate(f, cache));
        }
        conf::Cache::No => {
            hide_loading();
            let fs = Filesystem::new();
            f(fs).unwrap();
        }
    }
}
