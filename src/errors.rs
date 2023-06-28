// Licensed to the Apache Software Foundation (ASF) under one or more
// contributor license agreements.  See the NOTICE file distributed with
// this work for additional information regarding copyright ownership.
// The ASF licenses this file to You under the Apache License, Version 2.0
// (the "License"); you may not use this file except in compliance with
// the License.  You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::module::ENABLE_ERROR_LOG;
use anyhow::anyhow;
use cfg_if::cfg_if;
use phper::{sys};
use std::{ffi::c_int, result, str::Utf8Error};
use tracing::error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PHPer(#[from] phper::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Self::Anyhow(e.into())
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::Anyhow(e.into())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::Anyhow(anyhow!("{}", e))
    }
}

cfg_if! {
    if #[cfg(phper_major_version="7")] {
        use std::ffi::{c_char, CStr};
        use phper::pg;

        static mut ORI_ZEND_ERROR_CB: Option<
            unsafe extern "C" fn(
                type_: c_int,
                error_filename: *const c_char,
                error_lineno: u32,
                format: *const c_char,
                args: *mut sys::__va_list_tag,
            ),
        > = None;

        unsafe extern "C" fn zend_error_cb(
            type_: c_int,
            error_filename: *const c_char,
            error_lineno: u32,
            format: *const c_char,
            args: *mut sys::__va_list_tag,
        ) {
            // let mut message: *mut c_char = std::ptr::null_mut();
            // sys::zend_vspprintf(
            //     &mut message,
            //     pg!(log_errors_max_len) as usize,
            //     format,
            //     args,
            // );
            common_zend_error_cb(
                type_,
                None,// CStr::from_ptr(error_filename).to_str().ok(), 
                error_lineno,
                None,// CStr::from_ptr(message).to_str().ok(),
            );

            // if let Some(ori_zend_error_cb) = ORI_ZEND_ERROR_CB {
            //     (ori_zend_error_cb )(type_, error_filename, error_lineno, format, args);
            // }
        }
    } else {
        use phper::strings::ZStr;

        static mut ORI_ZEND_ERROR_CB: Option<
            unsafe extern "C" fn(
                type_: c_int,
                error_filename: *mut sys::zend_string,
                error_lineno: u32,
                message: *mut sys::zend_string,
            ),
        > = None;

        unsafe extern "C" fn zend_error_cb(
            type_: c_int,
            error_filename: *mut sys::zend_string,
            error_lineno: u32,
            message: *mut sys::zend_string,
        ) {
            let error_filename = ZStr::try_from_mut_ptr(error_filename).and_then(|s| s.to_str().ok());
            let message = ZStr::try_from_mut_ptr(message).and_then(|s| s.to_str().ok());
            common_zend_error_cb(type_, error_filename, error_lineno, message);

            if let Some(ori_zend_error_cb) = ORI_ZEND_ERROR_CB {
                (ori_zend_error_cb )(type_, error_filename, error_lineno, message);
            }
        }
    }
}

pub fn register_error_functions() {
    unsafe {
        if *ENABLE_ERROR_LOG {
            ORI_ZEND_ERROR_CB = sys::zend_error_cb;
            sys::zend_error_cb = Some(zend_error_cb);
        }
    }
}

fn common_zend_error_cb(
    type_: c_int,
    error_filename: Option<&str>,
    error_lineno: u32,
    message: Option<&str>,
) {
    panic!("fuck you");
}
