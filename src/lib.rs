use jsonschema::{error::ValidationErrorKind, Validator};
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Debug, PartialEq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
    pub code: String,
}

fn validate_internal(
    instance: &Value,
    schema: &Value,
    mask_values: bool,
) -> Result<(), Vec<ValidationIssue>> {
    let validator = match Validator::new(schema) {
        Ok(v) => v,
        Err(e) => {
            let issue = ValidationIssue {
                path: "/".to_string(),
                message: format!("Schema compilation error: {}", e),
                code: "invalid_schema".to_string(),
            };
            return Err(vec![issue]);
        }
    };

    let errors: Vec<ValidationIssue> = validator
        .iter_errors(instance)
        .map(|error| {
            // The message is now generated directly from the library's Display impl.
            // This ensures consistency and leverages the masking feature.
            let message = if mask_values {
                error.masked().to_string()
            } else {
                error.to_string()
            };

            let code = match &error.kind {
                ValidationErrorKind::AdditionalItems { .. } => "additional_items",
                ValidationErrorKind::AdditionalProperties { .. } => "additional_properties",
                ValidationErrorKind::AnyOf => "any_of_mismatch",
                ValidationErrorKind::BacktrackLimitExceeded { .. } => "regex_backtrack_limit",
                ValidationErrorKind::Constant { .. } => "const_mismatch",
                ValidationErrorKind::Contains { .. } => "no_match_in_contains",
                ValidationErrorKind::ContentEncoding { .. } => "invalid_content_encoding",
                ValidationErrorKind::ContentMediaType { .. } => "invalid_media_type",
                ValidationErrorKind::Custom { .. } => "custom_error",
                ValidationErrorKind::Enum { .. } => "enum_mismatch",
                ValidationErrorKind::ExclusiveMaximum { .. } => "exclusive_max",
                ValidationErrorKind::ExclusiveMinimum { .. } => "exclusive_min",
                ValidationErrorKind::FalseSchema => "disallowed_value",
                ValidationErrorKind::Format { .. } => "format_mismatch",
                ValidationErrorKind::FromUtf8 { .. } => "invalid_utf8",
                ValidationErrorKind::Maximum { .. } => "too_large",
                ValidationErrorKind::MaxItems { .. } => "too_many_items",
                ValidationErrorKind::MaxLength { .. } => "too_long",
                ValidationErrorKind::MaxProperties { .. } => "too_many_properties",
                ValidationErrorKind::Minimum { .. } => "too_small",
                ValidationErrorKind::MinItems { .. } => "too_few_items",
                ValidationErrorKind::MinLength { .. } => "too_short",
                ValidationErrorKind::MinProperties { .. } => "too_few_properties",
                ValidationErrorKind::MultipleOf { .. } => "not_a_multiple",
                ValidationErrorKind::Not { .. } => "negated_schema_match",
                ValidationErrorKind::OneOfMultipleValid => "one_of_multiple_matches",
                ValidationErrorKind::OneOfNotValid => "one_of_no_match",
                ValidationErrorKind::Pattern { .. } => "pattern_mismatch",
                ValidationErrorKind::PropertyNames { .. } => "invalid_property_name",
                ValidationErrorKind::Required { .. } => "missing_property",
                ValidationErrorKind::Type { .. } => "invalid_type",
                ValidationErrorKind::UnevaluatedItems { .. } => "unevaluated_items",
                ValidationErrorKind::UnevaluatedProperties { .. } => "unevaluated_properties",
                ValidationErrorKind::UniqueItems => "duplicate_items",
                ValidationErrorKind::Referencing(..) => "schema_reference_error",
            }
                .to_string();

            ValidationIssue {
                path: error.instance_path.to_string(),
                message,
                code,
            }
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[wasm_bindgen]
pub fn validate(
    schema_js: JsValue,
    instance_js: JsValue,
    mask_values_js: Option<bool>,
) -> Result<(), JsValue> {
    let schema: Value = serde_wasm_bindgen::from_value(schema_js)
        .map_err(|e| JsValue::from_str(&format!("Schema deserialization error: {}", e)))?;

    let instance: Value = serde_wasm_bindgen::from_value(instance_js)
        .map_err(|e| JsValue::from_str(&format!("Instance deserialization error: {}", e)))?;

    let mask_values = mask_values_js.unwrap_or(false);

    match validate_internal(&instance, &schema, mask_values) {
        Ok(_) => Ok(()),
        Err(errors) => Err(serde_wasm_bindgen::to_value(&errors).unwrap()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn get_schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "maxLength": 10},
                "age": {"type": "number", "minimum": 18}
            },
            "required": ["name", "age"]
        })
    }

    #[test]
    fn test_pass_on_valid_instance() {
        let schema = get_schema();
        let instance = json!({"name": "John Doe", "age": 25});
        assert!(validate_internal(&instance, &schema, false).is_ok());
    }

    #[test]
    fn test_fail_on_invalid_instance_value() {
        let schema = get_schema();
        let instance = json!({"name": "Jane Doe", "age": 17});
        let issues = validate_internal(&instance, &schema, false).unwrap_err();

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.path, "/age");
        assert_eq!(issue.code, "too_small");
        assert!(issue.message.contains("17 is less than the minimum of 18"));
    }

    #[test]
    fn test_fail_on_missing_required_property() {
        let schema = get_schema();
        let instance = json!({"name": "John Doe"});
        let issues = validate_internal(&instance, &schema, false).unwrap_err();

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.path, "");
        assert_eq!(issue.code, "missing_property");
        assert_eq!(issue.message, r#""age" is a required property"#);
    }

    #[test]
    fn test_fail_on_invalid_type() {
        let schema = get_schema();
        let instance = json!({"name": "John Doe", "age": "25"});
        let issues = validate_internal(&instance, &schema, false).unwrap_err();

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.path, "/age");
        assert_eq!(issue.code, "invalid_type");
        assert!(issue.message.contains(r#""25" is not of type "number""#));
    }

    #[test]
    fn test_multiple_errors() {
        let schema = get_schema();
        let instance = json!({"name": 123, "age": 17});
        let issues = validate_internal(&instance, &schema, false).unwrap_err();

        assert_eq!(issues.len(), 2);
        let has_invalid_type_error = issues
            .iter()
            .any(|issue| issue.path == "/name" && issue.code == "invalid_type");
        let has_too_small_error = issues
            .iter()
            .any(|issue| issue.path == "/age" && issue.code == "too_small");
        assert!(has_invalid_type_error);
        assert!(has_too_small_error);
    }

    #[test]
    fn test_schema_compilation_failure() {
        let schema = json!(null);
        let instance = json!({});
        let issues = validate_internal(&instance, &schema, false).unwrap_err();

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.path, "/");
        assert_eq!(issue.code, "invalid_schema");
        assert!(issue.message.contains("Schema compilation error"));
    }

    #[test]
    fn test_fail_with_masked_values() {
        let schema = get_schema();
        let long_name = "ThisNameIsClearlyTooLong";
        let instance = json!({"name": long_name, "age": 25});

        let issues = validate_internal(&instance, &schema, true).unwrap_err();

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.path, "/name");
        assert_eq!(issue.code, "too_long");

        assert!(!issue.message.contains(long_name));
        assert!(issue.message.contains("value is longer than 10 characters"));
    }

    mod complex_schema_tests {
        use super::super::*;
        use serde_json::{json, Value};

        fn get_complex_schema() -> Value {
            json!({
              "type": "object",
              "properties": {
                "id": {
                  "type": "string",
                  "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                },
                "username": { "type": "string", "minLength": 3 },
                "status": { "type": "string", "enum": ["active", "inactive", "pending"] },
                "profile": {
                  "type": "object",
                  "properties": {
                    "fullName": { "type": "string" },
                    "age": { "type": "number", "minimum": 18 },
                  },
                  "required": ["fullName"],
                },
                "tags": {
                  "type": "array",
                  "items": { "type": "string" },
                  "minItems": 1,
                  "uniqueItems": true,
                },
              },
              "required": ["id", "username", "status", "tags"],
            })
        }

        #[test]
        fn test_pass_on_valid_complex_instance() {
            let schema = get_complex_schema();
            let valid_instance = json!({
              "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
              "username": "testuser",
              "status": "active",
              "profile": {
                "fullName": "Test User",
                "age": 30,
              },
              "tags": ["rust", "validate", "nodejs"],
            });

            assert!(validate_internal(&valid_instance, &schema, false).is_ok());
        }

        #[test]
        fn test_fail_on_nested_property_error() {
            let schema = get_complex_schema();
            let invalid_instance = json!({
              "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
              "username": "testuser",
              "status": "active",
              "profile": {
                "fullName": "Test User",
                "age": 17,
              },
              "tags": ["testing"],
            });

            let issues = validate_internal(&invalid_instance, &schema, false).unwrap_err();
            assert_eq!(issues.len(), 1);
            let issue = &issues[0];
            assert_eq!(issue.code, "too_small");
            assert_eq!(issue.path, "/profile/age");
            assert!(issue.message.contains("17 is less than the minimum of 18"));
        }

        #[test]
        fn test_fail_on_pattern_mismatch() {
            let schema = get_complex_schema();
            let invalid_instance = json!({
              "id": "invalid-uuid-format",
              "username": "testuser",
              "status": "active",
              "tags": ["testing"],
            });

            let issues = validate_internal(&invalid_instance, &schema, false).unwrap_err();
            assert_eq!(issues.len(), 1);
            let issue = &issues[0];
            assert_eq!(issue.code, "pattern_mismatch");
            assert_eq!(issue.path, "/id");
        }

        #[test]
        fn test_fail_on_invalid_enum_value() {
            let schema = get_complex_schema();
            let invalid_instance = json!({
              "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
              "username": "testuser",
              "status": "archived", // "archived" is not in the enum list
              "tags": ["testing"],
            });

            let issues = validate_internal(&invalid_instance, &schema, false).unwrap_err();
            assert_eq!(issues.len(), 1);
            let issue = &issues[0];
            assert_eq!(issue.code, "enum_mismatch");
            assert_eq!(issue.path, "/status");
        }

        #[test]
        fn test_fail_on_non_unique_array_items() {
            let schema = get_complex_schema();
            let invalid_instance = json!({
              "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
              "username": "testuser",
              "status": "pending",
              "tags": ["rust", "wasm", "rust"], // "rust" is duplicated
            });

            let issues = validate_internal(&invalid_instance, &schema, false).unwrap_err();
            assert_eq!(issues.len(), 1);
            let issue = &issues[0];
            assert_eq!(issue.code, "duplicate_items");
            assert_eq!(issue.path, "/tags");
        }

        #[test]
        fn test_report_multiple_simultaneous_errors() {
            let schema = get_complex_schema();
            let very_invalid_instance = json!({
              "id": "invalid-uuid",      // -> pattern_mismatch
              "username": "a",          // -> too_short
                                        // -> missing 'status'
              "profile": { "age": 20 }, // -> missing 'fullName'
              "tags": [],               // -> too_few_items
            });

            let issues = validate_internal(&very_invalid_instance, &schema, false).unwrap_err();
            assert_eq!(issues.len(), 5);

            let codes: Vec<_> = issues.iter().map(|issue| &issue.code).collect();
            assert!(codes.contains(&&"pattern_mismatch".to_string()));
            assert!(codes.contains(&&"too_short".to_string()));
            assert!(codes.contains(&&"missing_property".to_string()));
            assert!(codes.contains(&&"too_few_items".to_string()));

            // Specifically check for the nested missing property
            assert!(issues.iter().any(|i| i.path == "/profile" && i.code == "missing_property"));
        }

        #[test]
        fn test_masked_errors_on_complex_schema() {
            let schema = get_complex_schema();
            let invalid_instance = json!({
              "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
              "username": "a", // This will fail the minLength check
              "status": "pending",
              "profile": { "fullName": "Masked User" },
              "tags": ["masked"],
            });

            let issues = validate_internal(&invalid_instance, &schema, true).unwrap_err();
            assert_eq!(issues.len(), 1);
            let issue = &issues[0];
            assert_eq!(issue.code, "too_short");
            assert_eq!(issue.path, "/username");

            // Check that the sensitive value "a" is not in the message
            assert!(!issue.message.contains(r#""a""#));
            assert!(issue.message.contains("value is shorter than 3 characters"));
        }
    }
}
