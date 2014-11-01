#![feature(globs)]
extern crate libc;

use xlib::{XOpenDisplay, XDisplayName, XQueryExtension, XInitExtension, XSynchronize};
use libc::{c_int};
use std::ptr;
use xtst::{XRecordCreateContext, XRecordAllClients, XRecordAllocRange, XRecordRange, XRecordQueryVersion, XRecordEnableContext, XRecordEnableContextAsync,XRecordInterceptData, XRecordProcessReplies, XRecordFreeData};
use std::mem;
mod xtst;
mod xlib;

fn main() {
	unsafe {
		let mut a:  i8 = 0;
		let display_control = XOpenDisplay( &a);
		let display_data = XOpenDisplay(&a);

		if display_data.is_null() || display_control.is_null() {
			fail!("XOpenDisplay() failed!");
		}

		XSynchronize(display_control, 1);
		
		let display_name = unsafe {XDisplayName(&a)};
		let ext_name = "RECORD";
		// Check presence of Record extension
		let arg2:*mut c_int = &mut 1;
		let arg3:*mut c_int = &mut 1;
		let arg4:*mut c_int = &mut 1;
		let has_record = XQueryExtension(display_control, ext_name.to_c_str().as_ptr() as *const i8,arg2,arg3,arg4);
		let extension = XInitExtension(display_control, ext_name.to_c_str().as_ptr() as *const i8);
		if extension.is_null() {
			fail!("XInitExtension() failed!");
		}

		// Get version
		let mut versionMajor: c_int = 0;
		let mut versionMinor: c_int = 0;
		XRecordQueryVersion(display_control, &mut versionMajor, &mut versionMinor);
		println!("RECORD extension version {}.{}", versionMajor, versionMinor);

		// Prepare record range
		let mut recordRange: XRecordRange = *XRecordAllocRange();
		let mut recordRangePtr: *mut *mut XRecordRange = std::mem::transmute(&mut &mut recordRange);
		recordRange.device_events.first = 2; // KeyPress
		recordRange.device_events.last = 6; // MotionNotify
		
		// Create context
		let context = XRecordCreateContext(
			display_control,
			0,
			&mut XRecordAllClients,
			1,
			recordRangePtr,
			1
		);
		if context == 0 {
			fail!("Fail create Record context\n");
		}

		// Run
		let res = XRecordEnableContextAsync(display_data, context, Some(recordCallback), &mut 0);
		if res == 0 {
			fail!("Cound not enable the Record context!\n");
		}
		loop {
			XRecordProcessReplies(display_data);
		}
	}
}

extern "C" fn recordCallback(pointer:*mut i8, raw_data: *mut XRecordInterceptData) {
	println!("Receive event\n");
	
	unsafe {
		let data = &*raw_data;
		println!("Category {}", data.category);
		println!("Time {}", data.server_time);
		println!("Datalen {}", data.data_len);
		XRecordFreeData(raw_data);
	}
}