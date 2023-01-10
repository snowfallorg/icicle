// Source: GNOME Social by Christopher Davis
// https://gitlab.gnome.org/World/Social

use gettextrs::gettext;
use gettextrs::ngettext;
use gettextrs::npgettext;
use gettextrs::pgettext;
use regex::Captures;
use regex::Regex;

#[allow(dead_code)]
fn freplace(input: String, args: &[&str]) -> String {
    let mut parts = input.split("{}");
    let mut output = parts.next().unwrap_or_default().to_string();
    for (p, a) in parts.zip(args.iter()) {
        output += &(a.to_string() + &p.to_string());
    }
    output
}

#[allow(dead_code)]
fn kreplace(input: String, kwargs: &[(&str, &str)]) -> String {
    let mut s = input;
    for (k, v) in kwargs {
        if let Ok(re) = Regex::new(&format!("\\{{{}\\}}", k)) {
            s = re
                .replace_all(&s, |_: &Captures<'_>| v.to_string())
                .to_string();
        }
    }
    s
}

// Simple translations functions

#[allow(dead_code)]
pub fn i18n(format: &str) -> String {
    gettext(format)
}

#[allow(dead_code)]
pub fn i18n_f(format: &str, args: &[&str]) -> String {
    let s = gettext(format);
    freplace(s, args)
}

#[allow(dead_code)]
pub fn i18n_k(format: &str, kwargs: &[(&str, &str)]) -> String {
    let s = gettext(format);
    kreplace(s, kwargs)
}

// Singular and plural translations functions

#[allow(dead_code)]
pub fn ni18n(single: &str, multiple: &str, number: u32) -> String {
    ngettext(single, multiple, number)
}

#[allow(dead_code)]
pub fn ni18n_f(single: &str, multiple: &str, number: u32, args: &[&str]) -> String {
    let s = ngettext(single, multiple, number);
    freplace(s, args)
}

#[allow(dead_code)]
pub fn ni18n_k(single: &str, multiple: &str, number: u32, kwargs: &[(&str, &str)]) -> String {
    let s = ngettext(single, multiple, number);
    kreplace(s, kwargs)
}

// Translations with context functions

#[allow(dead_code)]
pub fn pi18n(ctx: &str, format: &str) -> String {
    pgettext(ctx, format)
}

#[allow(dead_code)]
pub fn pi18n_f(ctx: &str, format: &str, args: &[&str]) -> String {
    let s = pgettext(ctx, format);
    freplace(s, args)
}

#[allow(dead_code)]
pub fn pi18n_k(ctx: &str, format: &str, kwargs: &[(&str, &str)]) -> String {
    let s = pgettext(ctx, format);
    kreplace(s, kwargs)
}

// Singular and plural with context

#[allow(dead_code)]
pub fn pni18n(ctx: &str, single: &str, multiple: &str, number: u32) -> String {
    npgettext(ctx, single, multiple, number)
}

#[allow(dead_code)]
pub fn pni18n_f(ctx: &str, single: &str, multiple: &str, number: u32, args: &[&str]) -> String {
    let s = npgettext(ctx, single, multiple, number);
    freplace(s, args)
}

#[allow(dead_code)]
pub fn pni18n_k(
    ctx: &str,
    single: &str,
    multiple: &str,
    number: u32,
    kwargs: &[(&str, &str)],
) -> String {
    let s = npgettext(ctx, single, multiple, number);
    kreplace(s, kwargs)
}
