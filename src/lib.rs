// bm25s for Android

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]
#![allow(non_snake_case)] // TODO: Delete me

extern crate libc;
use libc::{c_char, c_void};
use std::ffi::{CStr, CString};
use std::ptr;
use std::collections::HashMap as FxHashMap;

#[derive(Debug, Clone)]
pub struct DocumentStats {
    doc_id: u32,
    doc_length: u32,
    term_freq: FxHashMap<String, u32>,
}

#[derive(Debug, Clone)]
pub struct Index {
    inverted_index: FxHashMap<String, Vec<u32>>,
    doc_stats: FxHashMap<u32, DocumentStats>,
    total_doc_lengths: u32,
    k: f64,
    b: f64,
}

#[repr(C)]
pub struct CDocumentStats {
    doc_id: u32,
    doc_length: u32,
    term_freq_len: u32,
    term_freq_keys: *const *const c_char,
    term_freq_values: *const u32,
}

#[repr(C)]
pub struct CIndex {
    inverted_index_len: u32,
    inverted_index_keys: *const *const c_char,
    inverted_index_values: *const *const u32,
    doc_stats_len: u32,
    doc_stats_keys: *const u32,
    doc_stats_values: *const *const CDocumentStats,
    total_doc_lengths: u32,
    k: f64,
    b: f64,
}

#[no_mangle]
pub extern "C" fn create_index() -> *mut Index {
    Box::into_raw(Box::new(Index::new()))
}

#[no_mangle]
pub extern "C" fn add_document(index: *mut Index, doc: *const c_char, doc_id: u32) {
    let index = unsafe { &mut *index };
    let doc = unsafe { CStr::from_ptr(doc).to_str().unwrap() };
    index.upsert(doc, doc_id);
}

#[no_mangle]
pub extern "C" fn remove_document(index: *mut Index, doc_id: u32) {
    let index = unsafe { &mut *index };
    index.delete(doc_id);
}

#[no_mangle]
pub extern "C" fn search_index(index: *mut Index, query: *const c_char, top_k: u32) -> *mut Vec<(OrderedFloat<f64>, u32)> {
    let index = unsafe { &mut *index };
    let query = unsafe { CStr::from_ptr(query).to_str().unwrap() };
    let results = index.search(query, top_k);
    Box::into_raw(Box::new(results))
}

#[no_mangle]
pub extern "C" fn get_document_stats(index: *mut Index, doc_id: u32) -> *const CDocumentStats {
    let index = unsafe { &*index };
    if let Some(stats) = index.doc_stats.get(&doc_id) {
        let term_freq_len = stats.term_freq.len() as u32;
        let term_freq_keys: Vec<*const c_char> = stats.term_freq.keys()
            .map(|key| CString::new(key.as_str()).unwrap().into_raw())
            .collect();
        let term_freq_values: Vec<u32> = stats.term_freq.values().cloned().collect();

        let c_stats = CDocumentStats {
            doc_id: stats.doc_id,
            doc_length: stats.doc_length,
            term_freq_len,
            term_freq_keys: term_freq_keys.as_ptr(),
            term_freq_values: term_freq_values.as_ptr(),
        };
        Box::into_raw(Box::new(c_stats))
    } else {
        ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn get_index(index: *mut Index) -> *const CIndex {
    let index = unsafe { &*index };

    let inverted_index_len = index.inverted_index.len() as u32;
    let inverted_index_keys: Vec<*const c_char> = index.inverted_index.keys()
        .map(|key| CString::new(key.as_str()).unwrap().into_raw())
        .collect();
    let inverted_index_values: Vec<*const u32> = index.inverted_index.values()
        .map(|values| values.as_ptr())
        .collect();

    let doc_stats_len = index.doc_stats.len() as u32;
    let doc_stats_keys: Vec<u32> = index.doc_stats.keys().cloned().collect();
    let doc_stats_values: Vec<*const CDocumentStats> = index.doc_stats.values()
        .map(|stats| {
            let term_freq_len = stats.term_freq.len() as u32;
            let term_freq_keys: Vec<*const c_char> = stats.term_freq.keys()
                .map(|key| CString::new(key.as_str()).unwrap().into_raw())
                .collect();
            let term_freq_values: Vec<u32> = stats.term_freq.values().cloned().collect();

            let c_stats = CDocumentStats {
                doc_id: stats.doc_id,
                doc_length: stats.doc_length,
                term_freq_len,
                term_freq_keys: term_freq_keys.as_ptr(),
                term_freq_values: term_freq_values.as_ptr(),
            };
            Box::into_raw(Box::new(c_stats))
        })
        .collect();

    let c_index = CIndex {
        inverted_index_len,
        inverted_index_keys: inverted_index_keys.as_ptr(),
        inverted_index_values: inverted_index_values.as_ptr(),
        doc_stats_len,
        doc_stats_keys: doc_stats_keys.as_ptr(),
        doc_stats_values: doc_stats_values.as_ptr(),
        total_doc_lengths: index.total_doc_lengths,
        k: index.k,
        b: index.b,
    };

    Box::into_raw(Box::new(c_index))
}