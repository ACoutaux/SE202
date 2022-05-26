//! Module builds image and color structures with associated functions

use core::ops::{Div, IndexMut};
use core::{
    ops::{Index, Mul},
    panic,
};

use crate::gamma;
use micromath::F32Ext;

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Implements Color structure functions and constants
impl Color {
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };

    /// Applies gamma correction to each r g b bytes
    pub fn gamma_correct(&self) -> Self {
        Color {
            r: gamma::gamma_correct(self.r),
            g: gamma::gamma_correct(self.g),
            b: gamma::gamma_correct(self.b),
        }
    }
}

/// Implements multiplication for color type objects
impl core::ops::Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let check_overflow = |pixel: u8| (pixel as f32 * rhs).max(0.0).min(255.0).round() as u8;
        Color {
            r: check_overflow(self.r),
            g: check_overflow(self.g),
            b: check_overflow(self.b),
        }
    }
}

/// Implements division for color type objects using mul implementation
impl core::ops::Div<f32> for Color {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        self * 1.0 / rhs
    }
}

#[repr(transparent)]
pub struct Image([Color; 64]);

/// Implements functions for Image structure
impl Image {
    /// Creates new image with one given color
    pub fn new_solid(color: Color) -> Self {
        Image([color; 64])
    }

    /// Returns a line color array for a given line indice
    pub fn row(&self, row: usize) -> &[Color] {
        &self.0[(row - 1) * 8..(row - 1) * 8 + 8]
    }

    /// Builds a gradient image from a given color
    pub fn gradient(color: Color) -> Self {
        let mut image_grad = Image::default();
        for line in 1..=8 {
            for col in 1..=8 {
                image_grad.index_mut((line, col)).r =
                    (color.r as f32).div(1.0 + (line * line + col) as f32) as u8;
                image_grad.index_mut((line, col)).g =
                    (color.g as f32).div(1.0 + (line * line + col) as f32) as u8;
                image_grad.index_mut((line, col)).b =
                    (color.b as f32).div(1.0 + (line * line + col) as f32) as u8;
            }
        }
        image_grad
    }
}

/// Implements default function for image type objects
impl Default for Image {
    fn default() -> Self {
        Image([Color::default(); 64])
    }
}

/// Implements index function for image type objects
impl core::ops::Index<(usize, usize)> for Image {
    type Output = Color;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[(index.0 - 1) * 8 + index.1 - 1]
    }
}

/// Implements mutable index function for image type objects
impl core::ops::IndexMut<(usize, usize)> for Image {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[(index.0 - 1) * 8 + index.1 - 1]
    }
}

/// Implements as_ref() function for image type objects
impl AsRef<[u8; 192]> for Image {
    fn as_ref(&self) -> &[u8; 192] {
        unsafe { core::mem::transmute(self) }
    }
}

/// Implements as_mut() function for image type objects
impl AsMut<[u8; 192]> for Image {
    fn as_mut(&mut self) -> &mut [u8; 192] {
        unsafe { core::mem::transmute(self) }
    }
}
