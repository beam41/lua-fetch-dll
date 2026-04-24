use mlua::ffi::{self as ffi, lua_Integer, lua_State};
use std::os::raw::{c_char, c_int};
use std::panic;

// Push nil + error string and return 2 (the Lua nil,err convention).
unsafe fn push_err(state: *mut lua_State, msg: &str) -> c_int {
    ffi::lua_pushnil(state);
    ffi::lua_pushlstring(state, msg.as_ptr() as *const c_char, msg.len());
    2
}

// Recursively push a serde_json::Value onto the Lua stack.
unsafe fn push_json(state: *mut lua_State, value: serde_json::Value) {
    match value {
        serde_json::Value::Null => ffi::lua_pushnil(state),
        serde_json::Value::Bool(b) => ffi::lua_pushboolean(state, b as c_int),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ffi::lua_pushinteger(state, i);
            } else {
                ffi::lua_pushnumber(state, n.as_f64().unwrap_or(f64::NAN));
            }
        }
        serde_json::Value::String(s) => {
            ffi::lua_pushlstring(state, s.as_ptr() as *const c_char, s.len());
        }
        serde_json::Value::Array(arr) => {
            ffi::lua_createtable(state, arr.len() as c_int, 0);
            for (i, v) in arr.into_iter().enumerate() {
                push_json(state, v);
                ffi::lua_rawseti(state, -2, (i + 1) as lua_Integer);
            }
        }
        serde_json::Value::Object(obj) => {
            ffi::lua_createtable(state, 0, obj.len() as c_int);
            for (k, v) in obj {
                ffi::lua_pushlstring(state, k.as_ptr() as *const c_char, k.len());
                push_json(state, v);
                ffi::lua_rawset(state, -3);
            }
        }
    }
}

fn do_fetch_get(state: *mut lua_State) -> c_int {
    unsafe {
        let url_ptr = ffi::lua_tolstring(state, 1, std::ptr::null_mut());
        if url_ptr.is_null() {
            return push_err(state, "fetch.get: expected string URL");
        }
        let url = std::ffi::CStr::from_ptr(url_ptr)
            .to_string_lossy()
            .into_owned();
        match ureq::get(&url).call() {
            Err(e) => push_err(state, &e.to_string()),
            Ok(r) => match r.into_json::<serde_json::Value>() {
                Err(e) => push_err(state, &e.to_string()),
                Ok(v) => {
                    push_json(state, v);
                    1
                }
            },
        }
    }
}

fn do_fetch_get_string(state: *mut lua_State) -> c_int {
    unsafe {
        let url_ptr = ffi::lua_tolstring(state, 1, std::ptr::null_mut());
        if url_ptr.is_null() {
            return push_err(state, "fetch.get_string: expected string URL");
        }
        let url = std::ffi::CStr::from_ptr(url_ptr)
            .to_string_lossy()
            .into_owned();
        match ureq::get(&url).call() {
            Err(e) => push_err(state, &e.to_string()),
            Ok(r) => match r.into_string() {
                Err(e) => push_err(state, &e.to_string()),
                Ok(s) => {
                    ffi::lua_pushlstring(state, s.as_ptr() as *const c_char, s.len());
                    1
                }
            },
        }
    }
}

unsafe extern "C-unwind" fn lua_fetch_get(state: *mut lua_State) -> c_int {
    match panic::catch_unwind(panic::AssertUnwindSafe(|| do_fetch_get(state))) {
        Ok(n) => n,
        Err(_) => push_err(state, "fetch.get: internal panic"),
    }
}

unsafe extern "C-unwind" fn lua_fetch_get_string(state: *mut lua_State) -> c_int {
    match panic::catch_unwind(panic::AssertUnwindSafe(|| do_fetch_get_string(state))) {
        Ok(n) => n,
        Err(_) => push_err(state, "fetch.get_string: internal panic"),
    }
}

/// Module entry point – called by Lua's require("fetch.core").
#[no_mangle]
pub unsafe extern "C" fn luaopen_fetch_core(state: *mut lua_State) -> c_int {
    ffi::lua_createtable(state, 0, 2);

    ffi::lua_pushcclosure(state, lua_fetch_get, 0);
    ffi::lua_setfield(state, -2, b"get\0".as_ptr() as *const c_char);

    ffi::lua_pushcclosure(state, lua_fetch_get_string, 0);
    ffi::lua_setfield(state, -2, b"get_string\0".as_ptr() as *const c_char);

    1
}
