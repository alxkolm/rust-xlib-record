#![feature(globs)]
extern crate libc;

use xlib::{XOpenDisplay, XDisplayName, XQueryExtension, XInitExtension, XSynchronize, XKeysymToString, XKeycodeToKeysym};
use libc::{c_int};
use std::ptr;
use xtst::{XRecordCreateContext, XRecordAllClients, XRecordAllocRange, XRecordRange, XRecordQueryVersion, XRecordEnableContext, XRecordEnableContextAsync,XRecordInterceptData, XRecordProcessReplies, XRecordFreeData};
use std::mem;
use std::c_str::CString;
mod xtst;
mod xlib;
mod xlibint;
mod xutil;

struct XRecordDatum {
  xtype: ::libc::c_uchar,
  event: ::xlib::XEvent,
  req:   ::xlibint::xResourceReq,
  reply: ::xlibint::xGenericReply,
  error: ::xlibint::xError,
  setup: ::xlibint::xConnSetupPrefix,
}

// let mut display_control: *mut xlib::Display  = std::mem::transmute(0);
// let mut display_data: *mut xlib::Display  = std::mem::transmute(0);
static mut display_control: *mut xlib::Display = 0 as *mut xlib::Struct__XDisplay;
static mut display_data: *mut xlib::Display = 0 as *mut xlib::Struct__XDisplay;
static mut event_count:u32 = 0;
fn main() {
	unsafe {
		let mut a:  i8 = 0;
		display_control = XOpenDisplay(&a);
		display_data = XOpenDisplay(&a);

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
		xtst::XRecordFreeContext(display_data, context);
		loop {
			XRecordProcessReplies(display_data);
		}
	}
}

extern "C" fn recordCallback(pointer:*mut i8, raw_data: *mut XRecordInterceptData) {

	unsafe {

		let data = &*raw_data;
		// println!("Category {}", data.category);
		if data.category != xtst::XRecordFromServer {
			return;
		}
		println!("Event count: {}", event_count);
		event_count += 1;
		println!("Time {}", data.server_time);
		println!("Datalen {}", data.data_len);
		let mut xdatum_ptr: *mut XRecordDatum = data.data as *mut XRecordDatum;
		let mut xdatum = &*xdatum_ptr;
		
		let mut event = xdatum.event;
		println!("Type {}", xdatum.xtype);

		// Catch key event
		if xdatum.xtype == xtst::KeyPress || xdatum.xtype == xtst::KeyRelease {
			// extract key code
			let raw_bytes: &mut [u8,..4] = std::mem::transmute(data.data);
			let c_char  = XKeysymToString(XKeycodeToKeysym(display_control, raw_bytes[1], 0));
			let c_string = CString::new(std::mem::transmute(c_char), false);
			let char: &str = c_string.as_str().unwrap();
			println!("Keycode: {}, Character {}", raw_bytes[1], char);

			
			// println!("nitems {}", (&*wm_name).nitems);
		}
		// get current focus window
		let current_window: *mut xlib::Window = &mut 0;
		let revert_to_return: *mut i32 = &mut 0;
		xlib::XGetInputFocus(display_control, current_window, revert_to_return);
		println!("revertToReturn {}", *revert_to_return);
		let mut j = 0u;
		let mut wm_name: *mut xutil::XTextProperty = std::mem::transmute(&mut j);
		let mut wm_name_true: &str = "";
		let mut tmp =0i8;
		let mut c_wm_name: CString = CString::new(&tmp,false);
		let mut res = 0;
		// res = xutil::XGetWMName(display_control, *current_window, wm_name);
		let mut i = 0u;
		while res == 0 && i < 2 {
			print!(".");
			println!("current_window {}", *current_window);
			res = xutil::XGetWMName(display_control, *current_window, wm_name);
			
			
			if res == 0 {
				println!("Move to parent");
				let mut root: xlib::Window = 0;
				let mut parent: xlib::Window = 0;
				let mut childrens: *mut xlib::Window = &mut 0u64;
				let mut nchildren: u32 = 0;
				let r2 = xlib::XQueryTree(display_control, *current_window, &mut root, &mut parent, &mut childrens, &mut nchildren);
				if r2 == 0 {
					print!("*");
				} else {
					println!("parent {}", parent);
					*current_window = parent;
				}
			}
			// get parent window
			i += 1;
		}
		println!("---");
		if res == 0 {
			println!("no wmname");
		} else {
			// extract name from XTextProperty
			println!("wmname found!");
			println!("format: {}", (*wm_name).format);
			if (*wm_name).format != 8 {
				fail!("");
			}
			println!("encoding: {}", (*wm_name).encoding);
			let encoding_name = xlib::XGetAtomName(display_control, (*wm_name).encoding);
			let encoding_name_string = CString::new(std::mem::transmute(encoding_name), false);
			println!("encoding name: {}", encoding_name_string.as_str().unwrap());
			let mut tmp2 = 0i8;
			let mut list: *mut *mut ::libc::c_char = std::mem::transmute(&mut &mut tmp2);
			let mut list_count: i32 = 0;
			let res3 = xutil::XmbTextPropertyToTextList(display_control, std::mem::transmute(wm_name), &mut list, &mut list_count);
			println!("convert result: {}", res3);
			if res3 == 0 {
				println!("list count {}", list_count);
				c_wm_name = CString::new(std::mem::transmute(*list), false);
				wm_name_true = c_wm_name.as_str().unwrap();
				println!("wm_name: {}", wm_name_true);
				xlib::XFreeStringList(list);
			} else {
				c_wm_name = CString::new(std::mem::transmute((*wm_name).value), false);
				// wm_name_true = c_wm_name.as_str().unwrap();
				println!("wm_name bytes: {}", c_wm_name.as_bytes());
				// ая -      27 45 76 208 239 32 27 37 71 226 128 162 27 37 64 32 45 32 83 117 98 108 105 109 101 32 84 101 120 116 32 40 85 78 82 69 71 73 83 84 69 82 69 68 41 0
				// бв -      27 45 76 209 210 32 27 37 71 226 128 162 27 37 64 32 45 32 83 117 98 108 105 109 101 32 84 101 120 116 32 40 85 78 82 69 71 73 83 84 69 82 69 68 41 0
				// длодлод - 27 45 76 212 219 222 212 219 222 212 32 27 37 71 226 128 162 27 37 64 32 45 32 83 117 98 108 105 109 101 32 84 101 120 116 32 40 85 78 82 69 71 73 83 84 69 82 69 68 41 0

				println!("wm_name: {}", c_wm_name);
				println!("wm_name len w/o null: {}", c_wm_name.len());
			}
			
		}

		XRecordFreeData(raw_data);
	}
	println!("\n");
}

// fn string_convert(string: *mut libc::c_char) -> &'static str{
// 	let temp = unsafe{CString::new(std::mem::transmute(string), true)};
// 	let a: &str = temp.as_str().unwrap();
// }