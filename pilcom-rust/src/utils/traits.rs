use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use ::rand::Rand;
use fields::field_gl::Fr as FGL;
use fields::Field;
use serde::{de::DeserializeOwned, ser::Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;


pub trait FieldExtension:
    From<FGL>
    + From<u64>
    + From<i32>
    + From<usize>
    + Debug
    + Hash
    + Copy
    + Clone
    + PartialEq
    + Eq
    + Default
    + Add<Output = Self>
    + AddAssign
    + Div<Output = Self>
    + DivAssign
    + Mul<Output = Self>
    + MulAssign
    + Neg<Output = Self>
    + Sub<Output = Self>
    + SubAssign
    + Rand
    + Display
    + Send
    + Sync
    + Field
    + Serialize
    + DeserializeOwned
{
    const ELEMENT_BYTES: usize;
    const IS_CANONICAL: bool = false;
    const ZERO: Self;
    const ONE: Self;

    const ZEROS: Self;
    const ONES: Self;
    const NEW_SIZE: u64 = 0;
    fn dim(&self) -> usize;
    fn from_vec(values: Vec<FGL>) -> Self;
    fn to_be(&self) -> FGL;
    fn as_elements(&self) -> Vec<FGL>;
    fn mul_scalar(&self, b: usize) -> Self;
    fn _eq(&self, rhs: &Self) -> bool;
    fn gt(&self, rhs: &Self) -> bool;
    fn geq(&self, rhs: &Self) -> bool;
    fn lt(&self, rhs: &Self) -> bool;
    fn leq(&self, rhs: &Self) -> bool;
    fn exp(&self, e_: usize) -> Self;
    fn inv(&self) -> Self;
    fn as_int(&self) -> u64;
    fn elements_as_bytes(elements: &[Self]) -> &[u8];
    fn as_bytes(&self) -> &[u8];
    // TODO: Add generate rand fields vector for test/dev.
    // fn rand_
    // (&self) -> &[u8];
}