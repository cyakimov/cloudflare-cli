use cloudflare::framework::response::ApiFailure;

// Format errors from the cloudflare-rs cli for printing.
// Optionally takes an argument for providing a function that maps error code numbers to
// helpful additional information about why someone is getting an error message and how to fix it.
pub fn format_error(e: ApiFailure, err_helper: Option<&dyn Fn(u16) -> &'static str>) -> String {
    match e {
        ApiFailure::Error(_, api_errors) => {
            let mut complete_err = "".to_string();
            for error in api_errors.errors {
                let error_msg = format!("Code {}: {}\n", error.code, error.message);

                if let Some(annotate_help) = err_helper {
                    let suggestion_text = annotate_help(error.code);
                    let help_msg = format!("{}\n", suggestion_text);
                    complete_err.push_str(&format!("{}{}", error_msg, help_msg));
                } else {
                    complete_err.push_str(&error_msg)
                }
            }
            complete_err.trim_end().to_string() // Trimming strings in place for String is apparently not a thing...
        }
        ApiFailure::Invalid(reqwest_err) => format!("Error: {}", reqwest_err),
    }
}
