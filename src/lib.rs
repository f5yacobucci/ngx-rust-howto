use ngx::ffi::{
    nginx_version, ngx_array_push, ngx_command_t, ngx_conf_t, ngx_http_core_module,
    ngx_http_handler_pt, ngx_http_module_t, ngx_http_phases_NGX_HTTP_ACCESS_PHASE,
    ngx_http_request_t, ngx_int_t, ngx_module_t, ngx_str_t, ngx_uint_t, NGX_CONF_TAKE1,
    NGX_HTTP_LOC_CONF, NGX_HTTP_MODULE, NGX_RS_HTTP_LOC_CONF_OFFSET, NGX_RS_MODULE_SIGNATURE,
};
use ngx::http::{HTTPModule, MergeConfigError};
use ngx::{core, core::Status, http};
use ngx::{http_request_handler, ngx_log_debug_http, ngx_modules, ngx_null_command, ngx_string};
use std::os::raw::{c_char, c_void};
use std::ptr::addr_of;

struct Module;

// Implement our HTTPModule trait, we're creating a postconfiguration method to install our
// handler's Access phase function.
impl http::HTTPModule for Module {
    type MainConf = ();
    type SrvConf = ();
    type LocConf = ModuleConfig;

    unsafe extern "C" fn postconfiguration(cf: *mut ngx_conf_t) -> ngx_int_t {
        let htcf = http::ngx_http_conf_get_module_main_conf(cf, &*addr_of!(ngx_http_core_module));

        let h = ngx_array_push(
            &mut (*htcf).phases[ngx_http_phases_NGX_HTTP_ACCESS_PHASE as usize].handlers,
        ) as *mut ngx_http_handler_pt;
        if h.is_null() {
            return core::Status::NGX_ERROR.into();
        }

        // set an Access phase handler
        *h = Some(howto_access_handler);
        core::Status::NGX_OK.into()
    }
}

// Create a ModuleConfig to save our configuration state.
#[derive(Debug, Default)]
struct ModuleConfig {
    enabled: bool,
    method: String,
}

// Implement our Merge trait to merge configuration with higher layers.
impl http::Merge for ModuleConfig {
    fn merge(&mut self, prev: &ModuleConfig) -> Result<(), MergeConfigError> {
        if prev.enabled {
            self.enabled = true;
        }

        if self.method.is_empty() {
            self.method = String::from(if !prev.method.is_empty() {
                &prev.method
            } else {
                ""
            });
        }

        if self.enabled && self.method.is_empty() {
            return Err(MergeConfigError::NoValue);
        }
        Ok(())
    }
}

// Create our "C" module context with function entrypoints for NGINX event loop. This "binds" our
// HTTPModule implementation to functions callable from C.
#[no_mangle]
static ngx_http_howto_module_ctx: ngx_http_module_t = ngx_http_module_t {
    preconfiguration: Some(Module::preconfiguration),
    postconfiguration: Some(Module::postconfiguration),
    create_main_conf: Some(Module::create_main_conf),
    init_main_conf: Some(Module::init_main_conf),
    create_srv_conf: Some(Module::create_srv_conf),
    merge_srv_conf: Some(Module::merge_srv_conf),
    create_loc_conf: Some(Module::create_loc_conf),
    merge_loc_conf: Some(Module::merge_loc_conf),
};

// Create our module structure and export it with the `ngx_modules!` macro. For this simple
// handler, the ngx_module_t is predominately boilerplate save for setting the above context into
// this structure and setting our custom configuration command (defined below).
ngx_modules!(ngx_http_howto_module);

#[no_mangle]
pub static mut ngx_http_howto_module: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::max_value(),
    index: ngx_uint_t::max_value(),
    name: std::ptr::null_mut(),
    spare0: 0,
    spare1: 0,
    version: nginx_version as ngx_uint_t,
    signature: NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,

    ctx: &ngx_http_howto_module_ctx as *const _ as *mut _,
    commands: unsafe { &ngx_http_howto_commands[0] as *const _ as *mut _ },
    type_: NGX_HTTP_MODULE as ngx_uint_t,

    init_master: None,
    init_module: None,
    init_process: None,
    init_thread: None,
    exit_thread: None,
    exit_process: None,
    exit_master: None,

    spare_hook0: 0,
    spare_hook1: 0,
    spare_hook2: 0,
    spare_hook3: 0,
    spare_hook4: 0,
    spare_hook5: 0,
    spare_hook6: 0,
    spare_hook7: 0,
};

// Register and allocate our command structures for directive generation and eventual storage. Be
// sure to terminate the array with the ngx_null_command! macro.
#[no_mangle]
static mut ngx_http_howto_commands: [ngx_command_t; 2] = [
    ngx_command_t {
        name: ngx_string!("howto"),
        type_: (NGX_HTTP_LOC_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_howto_commands_set_method),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_null_command!(),
];

#[no_mangle]
extern "C" fn ngx_http_howto_commands_set_method(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);
        let args = (*(*cf).args).elts as *mut ngx_str_t;
        conf.enabled = true;
        conf.method = (*args.add(1)).to_string();
    };

    std::ptr::null_mut()
}

// Implement a request handler. Use the convenience macro, the http_request_handler! macro will
// convert the native NGINX request into a Rust Request instance as well as define an extern C
// function callable from NGINX.
//
// The function body is implemented as a Rust closure.
http_request_handler!(howto_access_handler, |request: &mut http::Request| {
    let co = unsafe { request.get_module_loc_conf::<ModuleConfig>(&*addr_of!(ngx_http_howto_module)) };
    let co = co.expect("module config is none");

    ngx_log_debug_http!(request, "howto module enabled called");

    match co.enabled {
        true => {
            let method = request.method();

            if method.as_str() == co.method {
                return core::Status::NGX_OK;
            }
            http::HTTPStatus::FORBIDDEN.into()
        }
        false => core::Status::NGX_OK,
    }
});
