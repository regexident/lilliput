use std::mem::transmute_copy;

use crate::floats::{F32, F64};

impl From<f32> for F32 {
    fn from(value: f32) -> Self {
        unsafe { transmute_copy(&value) }
    }
}

impl From<F32> for f32 {
    fn from(value: F32) -> Self {
        unsafe { transmute_copy(&value) }
    }
}

impl From<f64> for F64 {
    fn from(value: f64) -> Self {
        unsafe { transmute_copy(&value) }
    }
}

impl From<F64> for f64 {
    fn from(value: F64) -> Self {
        unsafe { transmute_copy(&value) }
    }
}
