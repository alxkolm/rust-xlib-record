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

			
			// println!("nitems {}", (&*wm_name).nitems);
		}
		// get current focus window
		// let current_window: *mut xlib::Window = &mut 0;
		// let revert_to_return: *mut i32 = &mut 0;
		let current_window = display_control.get_input_focus();
		// println!("revertToReturn {}", *revert_to_return);
		let mut j = 0u;
		let mut wm_name: *mut xutil::XTextProperty = std::mem::transmute(&mut j);
		let mut wm_name_true: &str = "";
		let mut tmp =0i8;
		let mut c_wm_name: CString = CString::new(&tmp,false);
		let mut res = 0;
		// res = xutil::XGetWMName(display_control.display, *current_window, wm_name);
		let mut i = 0u;
		while res == 0 && i < 2 {
			print!(".");
			println!("current_window {}", *current_window);
			if current_window.id == 0 {
				break;
			}
			res = xutil::XGetWMName(display_control.display, *current_window, wm_name);
			match current_window.get_wm_name() {
				None => {
					println!("Move to parent");
					let mut root: xlib::Window = 0;
					let mut parent: xlib::Window = 0;
					let mut childrens: *mut xlib::Window = &mut 0u64;
					let mut nchildren: u32 = 0;
					let r2 = xlib::XQueryTree(display_control.display, *current_window, &mut root, &mut parent, &mut childrens, &mut nchildren);
					if r2 == 0 {
						print!("*");
					} else {
						println!("parent {}", parent);
						*current_window = parent;
					}
				}
			}
			
			
			if res == 0 {
				// println!("Move to parent");
				// let mut root: xlib::Window = 0;
				// let mut parent: xlib::Window = 0;
				// let mut childrens: *mut xlib::Window = &mut 0u64;
				// let mut nchildren: u32 = 0;
				// let r2 = xlib::XQueryTree(display_control.display, *current_window, &mut root, &mut parent, &mut childrens, &mut nchildren);
				// if r2 == 0 {
				// 	print!("*");
				// } else {
				// 	println!("parent {}", parent);
				// 	*current_window = parent;
				// }
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
				panic!("");
			}
			println!("encoding: {}", (*wm_name).encoding);
			let encoding_name = xlib::XGetAtomName(display_control.display, (*wm_name).encoding);
			let encoding_name_string = CString::new(std::mem::transmute(encoding_name), false);
			println!("encoding name: {}", encoding_name_string.as_str().unwrap());
			let mut tmp2 = 0i8;
			let mut list: *mut *mut ::libc::c_char = std::mem::transmute(&mut &mut tmp2);
			let mut list_count: i32 = 0;
			let res3 = xutil::XmbTextPropertyToTextList(display_control.display, std::mem::transmute(wm_name), &mut list, &mut list_count);
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
				// zx -      122 120 32 27 37 71 226 128 162 27 37 64 32 45 32 83 117 98 108 105 109 101 32 84 101 120 116 32 40 85 78 82 69 71 73 83 84 69 82 69 68 41 0
				// длодлод - 27 45 76 212 219 222 212 219 222 212 32 27 37 71 226 128 162 27 37 64 32 45 32 83 117 98 108 105 109 101 32 84 101 120 116 32 40 85 78 82 69 71 73 83 84 69 82 69 68 41 0
				// zx • - Sublime Text (UNREGISTERED)

				println!("wm_name: {}", c_wm_name);
				println!("wm_name len w/o null: {}", c_wm_name.len());
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
		Window {id: 0, display: self}
	}
}

struct Window<'a> {
    id: u64, // XID
    display: &'a Display<'a>
}

impl<'a> Window<'a> {
	fn get_wm_name(&self) -> Option<String> {
		let mut a:String = String::new();
		let wmname = unsafe {
			let mut window_name: *mut i8 = 0 as *mut i8;
			let res = xlib::XFetchName(self.display.display, self.id, &mut window_name);
			if res != 0 {
				let c_wm_name = CString::new(std::mem::transmute(window_name), false);
				// xlib::XFree(&mut window_name);
				Some(String::from_str(c_wm_name.as_str().unwrap()))

			} else {
				None
			}
		};
		wmname
	}
	fn get_tree (&self) -> Option<WindowTree> {
		unsafe {
			let mut root: xlib::Window = 0;
			let mut parent: xlib::Window = 0;
			let mut children: *mut xlib::Window = &mut 0u64;
			let mut nchildren: u32 = 0;

			let res = xlib::XQueryTree(
				self.display.display,
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
							let b: Vec<Window> = Vec::new();
							for i in range(0, nchildren as int){
								b.push(Window{id: *children.offset(i), display: self.display});
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