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
// static mut display_control: *mut xlib::Display = 0 as *mut xlib::Struct__XDisplay;
// static mut display_data: *mut xlib::Display = 0 as *mut xlib::Struct__XDisplay;
static mut display_control: Display<'static> = Display {display: 0 as *mut xlib::Display};
static mut display_data: Display<'static> = Display {display: 0 as *mut xlib::Display};
static mut event_count:u32 = 0;
fn main() {
	unsafe {
		let mut a:  i8 = 0;
		display_control = Display::new();
		display_data = Display::new();

		XSynchronize(display_control.display, 1);

		let display_name = unsafe {XDisplayName(&a)};
		let ext_name = "RECORD";
		// Check presence of Record extension
		let arg2:*mut c_int = &mut 1;
		let arg3:*mut c_int = &mut 1;
		let arg4:*mut c_int = &mut 1;
		let has_record = XQueryExtension(display_control.display, ext_name.to_c_str().as_ptr() as *const i8,arg2,arg3,arg4);
		let extension = XInitExtension(display_control.display, ext_name.to_c_str().as_ptr() as *const i8);
		if extension.is_null() {
			panic!("XInitExtension() failed!");
		}

		// Get version
		let mut versionMajor: c_int = 0;
		let mut versionMinor: c_int = 0;
		XRecordQueryVersion(display_control.display, &mut versionMajor, &mut versionMinor);
		println!("RECORD extension version {}.{}", versionMajor, versionMinor);

		// Prepare record range
		let mut recordRange: XRecordRange = *XRecordAllocRange();
		let mut recordRangePtr: *mut *mut XRecordRange = std::mem::transmute(&mut &mut recordRange);
		recordRange.device_events.first = 2; // KeyPress
		recordRange.device_events.last = 6; // MotionNotify
		
		// Create context
		let context = XRecordCreateContext(
			display_control.display,
			0,
			&mut XRecordAllClients,
			1,
			recordRangePtr,
			1
		);
		if context == 0 {
			panic!("Fail create Record context\n");
		}

		// Run
		let res = XRecordEnableContextAsync(display_data.display, context, Some(recordCallback), &mut 0);
		if res == 0 {
			panic!("Cound not enable the Record context!\n");
		}
		xtst::XRecordFreeContext(display_data.display, context);
		loop {
			XRecordProcessReplies(display_data.display);
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
			let c_char  = XKeysymToString(XKeycodeToKeysym(display_control.display, raw_bytes[1], 0));
			let c_string = CString::new(std::mem::transmute(c_char), false);
			let char: &str = c_string.as_str().unwrap();
			println!("Keycode: {}, Character {}", raw_bytes[1], char);
		}
		
		let mut current_window = display_control.get_input_focus();
		let mut parent_window: Option<Window> = None;
		let mut wm_name_str: Option<String> = None;
		
		let mut i = 0u;
		while i < 10 {
			if current_window.id == 0 {
				break;
			}
			
			wm_name_str = current_window.get_wm_name();
			match wm_name_str {
				None => {
					let tree = current_window.get_tree();
					parent_window = match tree {
						Some(WindowTree{parent: parent, children: _}) => {
							Some(parent)
						},
						_ => None
					}
					
				},
				Some(ref wmname) => {
					// wm_name_str = Some(wmname);
					break;
				}
			}
			
			current_window = match parent_window {
				Some(win) => win,
				_ => current_window
			};
			
			i += 1;
		}
		match wm_name_str {
			Some(ref name) => {
				println!("WM_NAME: {}", *name);
			},
			None => {
				println!("WM_NAME: none");
			}
		}

		XRecordFreeData(raw_data);
	}
	println!("\n");
}


// ============================================================================
// Simple naive wrappers around X stuff
// ============================================================================
struct Display<'a> {
    display: *mut xlib::Display,
}

impl<'a> Display<'a> {
	fn new() -> Display<'a> {
		Display {display: unsafe {
			let mut a:  i8 = 0;
			let dpy = XOpenDisplay(&a);
			if dpy.is_null() {
				panic!("XOpenDisplay() failed!");
			}
			dpy
		}}
	}
	fn get_input_focus(&self) -> Window{
		let current_window: *mut xlib::Window = &mut 0;
		let revert_to_return: *mut i32 = &mut 0;
		unsafe{xlib::XGetInputFocus(self.display, current_window, revert_to_return)};
		Window {id: unsafe{*current_window}, display: self.display}
	}
}

struct Window<'a> {
    id: u64, // XID
    display: *mut xlib::Display
}

impl<'a> Window<'a> {
	fn get_wm_name(&self) -> Option<String> {
		let mut a:String = String::new();
		let wmname = unsafe {
			let mut window_name: *mut i8 = 0 as *mut i8;
			let res = xlib::XFetchName(self.display, self.id, &mut window_name);
			if res != 0 {
				let c_wm_name = CString::new(std::mem::transmute(window_name), false);
				// xlib::XFree(&mut window_name);
				Some(String::from_str(c_wm_name.as_str().unwrap()))
			} else {
				// Try get _NET_WM_NAME
				None
			}
		};
		wmname
	}
	fn get_property(&self, property_name: &str, property_type: &str) -> Option<CVec>{
		unsafe {
			let xa_property_type: xlibint::Atom = xlib::XInternAtom(self.display, property_type.to_c_str().as_ptr(), 0);
			let xa_property_name: xlibint::Atom = xlib::XInternAtom(self.display, property_name.to_c_str().as_ptr(), 0)
			let mut actual_type_return  : xlibint::Atom     = 0;
			let mut actual_format_return: libc::c_int       = 0;
			let mut nitems_return       : libc::c_ulong     = 0;
			let mut bytes_after_return  : libc::c_ulong     = 0;
			let mut prop_return         : *mut libc::c_char = 0;
			let res = xlib::XGetWindowProperty(
				self.display,
				self.id,
				atom,
				0,
				4096 / 4,
				0,
				xa_property_type,
				&mut actual_type_return,
				&mut actual_format_return,
				&mut nitems_return,
				&mut bytes_after_return,
				&mut prop_return
				);
			if (xa_property_type != actual_type_return) {
				println!("Invalid type of {} property", property_name);
				return None;
			}
			let tmp_size = (actual_format_return / 8) * nitems_return;
			let data = c_vec::CVec::new(prop_return, tmp_size);
			Some(data);
		}
	}
	
	fn get_tree (&self) -> Option<WindowTree> {
		unsafe {
			let mut root: xlib::Window = 0;
			let mut parent: xlib::Window = 0;
			let mut children: *mut xlib::Window = &mut 0u64;
			let mut nchildren: u32 = 0;

			let res = xlib::XQueryTree(
				self.display,
				self.id,
				&mut root,
				&mut parent,
				&mut children,
				&mut nchildren);

			match res {
				0 => None,
				_ => {
					let childs = match nchildren {
						0 => None,
						_ => {
							// let c = std::c_vec::CVec::new(children, nchildren);
							let mut b: Vec<Window> = Vec::new();
							for i in range(0, nchildren as int){
								b.push(Window{
									id: *children.offset(i),
									display: self.display
								});
							}
							Some(b)
						}
					};

					Some(WindowTree {
						parent: Window{
							id: parent,
							display: self.display,
						},
						children: childs
					})
				}
			}
		}
	}
}

struct WindowTree<'a> {
    parent: Window<'a>,
    children: Option<Vec<Window<'a>>>,
}