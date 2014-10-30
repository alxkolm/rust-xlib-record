#![feature(globs)]
extern crate xlib;
extern crate libc;

use xlib::{XOpenDisplay, XDisplayName, XQueryExtension, XInitExtension};
use libc::{c_int};
use std::ptr;
use xtst::{XRecordCreateContext, XRecordAllClients, XRecordAllocRange, XRecordRange};
use std::mem;
mod xtst;



fn main() {
	let mut a:  i8 = 0;
	let a_ptr: *mut i8 = &mut a;

	let display = unsafe { XOpenDisplay(a_ptr)};
	if display.is_null() {
		fail!("XOpenDisplay() failed!");
	}

	let display_name = unsafe {XDisplayName(a_ptr)};
	let ext_name = "RECORD";

	let arg2:*mut c_int = &mut 1;
	let arg3:*mut c_int = &mut 1;
	let arg4:*mut c_int = &mut 1;
	let has_record = unsafe{XQueryExtension(display, ext_name.to_c_str().as_ptr() as *mut i8,arg2,arg3,arg4)};
	let extension = unsafe{XInitExtension(display, ext_name.to_c_str().as_ptr() as *mut i8)};
	if extension.is_null() {
		fail!("XInitExtension() failde!");
	}

	unsafe {
		
		let mut recordRange: *mut XRecordRange = XRecordAllocRange();
		(&*recordRange).device_events.first = 2; // KeyPress
		(&*recordRange).device_events.last = 6; // MotionNotify
		let context = XRecordCreateContext(
			display,
			0,
			&mut XRecordAllClients,
			0,
			&mut recordRange,
			0
		);
	}
	
	println!("display: {}\ndisplay name: {}\nRECORD extension:{}", display, display_name, has_record);
}