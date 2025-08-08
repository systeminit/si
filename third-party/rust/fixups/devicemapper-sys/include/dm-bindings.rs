// Minimal bindings for devicemapper-sys remote builds
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub type dm_task = *mut ::std::os::raw::c_void;

extern "C" {
    pub fn dm_task_create(type_: ::std::os::raw::c_int) -> *mut dm_task;
    pub fn dm_task_destroy(dmt: *mut dm_task);
    pub fn dm_task_run(dmt: *mut dm_task) -> ::std::os::raw::c_int;
}