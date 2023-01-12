use do_notation::m;
use js_sys::JSON;
use snafu::{ensure, ResultExt, Snafu};
use wasm_bindgen::JsValue;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(
        "serde wasm parsing errored: {the_falied_to_parse_value} - source: {source}"
    ))]
    SerdeWasmBindgenCouldNotParseResponse {
        the_falied_to_parse_value: String,
        source: serde_wasm_bindgen::Error,
    },

    #[snafu(display("The slack response was not ok: {response}"))]
    SlackResponseNotOk { response: String },

    #[snafu(display("Provided string value could not be parsed to json: {string}"))]
    CouldNotParseJsonFromString { string: String },

    #[snafu(display(
        "The javascript value was not a string when it should have been: {js_value}"
    ))]
    JsValueNotAString { js_value: String },

    #[snafu(display(
        "Tried to parse a javascript value that was not an object into an object. value: {the_failed_to_parse_value} - type: {type}"
    ))]
    JsValueWasNotObject {
        the_failed_to_parse_value: String,
        r#type: String,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;
pub trait SlackResponseValidator {
    fn ok(&self) -> Option<bool>;

    fn validate_response(self) -> Result<Self>
    where
        Self: Sized,
        Self: std::fmt::Debug,
    {
        ensure!(
            self.ok().expect(
                "Expected ok field to have a value in the response, but got None. This is a bug"
            ),
            SlackResponseNotOkSnafu {
                response: format!("{:#?}", self),
            }
        );
        Ok(self)
    }
}

pub fn convert_result_string_to_object(val: JsValue) -> Result<JsValue> {
    // results from the `request` function of obsidian return strings
    m! {
        str_val <- val
                   .as_string()
                   .map_or(JsValueNotAStringSnafu {js_value: format!("{:#?}", val)}.fail(), Ok);
        // mapping error instead of using snafu context because jsvalue is not an Error from parse method
        obj_val <- JSON::parse(&str_val).map_err(|_err| Error::CouldNotParseJsonFromString {string: format!("{:#?}", str_val)});
        return obj_val;
    }
}

pub fn defined_from_js_object<T>(val: JsValue) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    ensure!(
        val.is_object(),
        JsValueWasNotObjectSnafu {
            the_failed_to_parse_value: format!("{:#?}", val),
            r#type: format!("{:#?}", val.js_typeof())
        }
    );
    let val_string = format!("{:#?}", &val);

    serde_wasm_bindgen::from_value(val).context(SerdeWasmBindgenCouldNotParseResponseSnafu {
        the_falied_to_parse_value: val_string,
    })
}
