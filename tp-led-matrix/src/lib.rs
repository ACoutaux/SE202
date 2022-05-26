//! Library module which makes available modules for whole project

#![no_std] //do not use standard library in an embedded context


pub mod gamma;
pub use image::{Color,Image};
pub mod image;
pub mod matrix;
