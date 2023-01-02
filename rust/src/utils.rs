use do_notation::m;
use js_sys::{Promise, JSON};
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::JsValue;

use crate::{errors::SlackError, request, slack_http_client::RequestUrlParam};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn convert_result_string_to_object(val: JsValue) -> Result<JsValue, SlackError> {
    // results from the `request` function of obsidian return strings
    m! {
        str_val <- val
                   .as_string()
                   .map_or(Err(SlackError::EmptyResult(format!("{:#?}", val))), Ok);
        obj_val <- JSON::parse(&str_val)
                   .map_err(|err| SlackError::ResponseNotAnObject(format!("{:#?} | {:#?}", err, val)));
        return obj_val;
    }
}

pub fn make_request(params: RequestUrlParam) -> Promise {
    let serializer = Serializer::json_compatible();
    request(params.serialize(&serializer).unwrap())
}
