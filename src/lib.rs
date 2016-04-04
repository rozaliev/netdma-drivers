#![feature(plugin)]
#![feature(question_mark)]
#![feature(volatile)]
#![feature(associated_consts)]

#![plugin(registers)]

extern crate netdma;
extern crate hugetlb;
extern crate netbuf;
extern crate libc;

pub use netdma::{PciAddr, PciDevice};

pub mod intel;
