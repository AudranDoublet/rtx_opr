#![feature(str_strip)]

extern crate proc_macro;
extern crate proc_macro2;

use std::fs::read_to_string;

use syn::Lit;
use quote::{quote, ToTokens};

struct TokenList<T> {
    list: Vec<T>,
}

impl<T: ToTokens> ToTokens for TokenList<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for i in &self.list {
            i.to_tokens(tokens);
        }
    }
}

fn read_shader_litteral(readed_files: &mut Vec<String>, path: &str) -> String {
    let s = read_file(path);

    readed_files.push(full_path(path));

    let mut result = "".to_string();

    for line in s.lines() {
        if let Some(file) = line.strip_prefix("#include ") {
            result += read_shader_litteral(readed_files, file).as_str();
        } else {
            result += line;
        }

        result += "\n";
    }

    result
}

fn stream_to_str(input: proc_macro2::TokenStream) -> String {
    let len = input.clone().into_iter().count();

    if len != 1 {
        panic!("argument must be a single string literal, but got {} tokens", len);
    }

    match syn::parse2::<Lit>(input) {
        Ok(Lit::Str(lit)) => lit.value(),
        Ok(_) | Err(_) => {
            panic!("argument must be a single string literal");
        }
    }
}

fn read_file(path: &str) -> String {
    match read_to_string(path) {
        Ok(v) => v,
        Err(e) => panic!("can't read shader `{}`: {}", path, e)
    }
}

fn full_path(file: &str) -> String {
    std::path::Path::new(file).canonicalize().unwrap().to_str().unwrap().to_string()
}

#[proc_macro]
pub fn read_shader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let file = stream_to_str(proc_macro2::TokenStream::from(input));
    let mut readed_files = vec![];

    let value = read_shader_litteral(&mut readed_files, &file);

    let iter = readed_files.iter();

    let parsed = quote!(
        {
            let __force_recompilation = vec![#(include_str!(#iter)),*];
            #value
        }
    );

    proc_macro::TokenStream::from(parsed)
}
