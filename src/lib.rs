use serde::{Deserialize, Serialize};
use jsonschema::{error::ValidationErrorKind, Validator};
use serde_json::Value;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
    pub code: String,
}

fn perform_validation(
    validator: &Validator,
    instance: &Value,
    mask_values: bool,
) -> Result<(), Vec<ValidationIssue>> {
    let errors: Vec<ValidationIssue> = validator
        .iter_errors(instance)
        .map(|error| {
            let message = if mask_values {
                error.masked().to_string()
            } else {
                error.to_string()
            };
            ValidationIssue {
                path: error.instance_path.to_string(),
                message,
                code: map_error_kind_to_code(&error.kind),
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
pub struct WasmValidator {
    validator: Validator,
    mask_values: bool,
}

#[wasm_bindgen]
impl WasmValidator {
    #[wasm_bindgen(constructor)]
    pub fn new(schema_js: JsValue, mask_values_js: Option<bool>) -> Result<WasmValidator, JsValue> {
        let schema: Value = serde_wasm_bindgen::from_value(schema_js)
            .map_err(|e| JsValue::from_str(&format!("Schema deserialization error: {}", e)))?;

        let validator = Validator::new(&schema).map_err(|e| {
            let issue = ValidationIssue {
                path: "/".to_string(),
                message: format!("Schema compilation error: {}", e),
                code: "invalid_schema".to_string(),
            };
            serde_wasm_bindgen::to_value(&vec![issue]).unwrap()
        })?;

        Ok(WasmValidator {
            validator,
            mask_values: mask_values_js.unwrap_or(false),
        })
    }

    #[wasm_bindgen]
    pub fn validate(&self, instance_js: JsValue) -> Result<(), JsValue> {
        let instance: Value = serde_wasm_bindgen::from_value(instance_js)
            .map_err(|e| JsValue::from_str(&format!("Instance deserialization error: {}", e)))?;

        match perform_validation(&self.validator, &instance, self.mask_values) {
            Ok(_) => Ok(()),
            Err(errors) => Err(serde_wasm_bindgen::to_value(&errors).unwrap()),
        }
    }
}

#[wasm_bindgen]
pub fn validate(
    schema_js: JsValue,
    instance_js: JsValue,
    mask_values_js: Option<bool>,
) -> Result<(), JsValue> {
    let validator = WasmValidator::new(schema_js, mask_values_js)?;
    validator.validate(instance_js)
}

fn map_error_kind_to_code(kind: &ValidationErrorKind) -> String {
    match kind {
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
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn get_complex_schema() -> Value {
        json!({
          "type": "object",
          "properties": {
            "id": { "type": "string", "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$" },
            "username": { "type": "string", "minLength": 3 },
            "status": { "type": "string", "enum": ["active", "inactive", "pending"] },
            "profile": {
              "type": "object",
              "properties": { "fullName": { "type": "string" }, "age": { "type": "number", "minimum": 18 } },
              "required": ["fullName"],
            },
            "tags": { "type": "array", "items": { "type": "string" }, "minItems": 1, "uniqueItems": true },
          },
          "required": ["id", "username", "status", "tags"],
        })
    }

    #[test]
    fn test_pass_on_valid_complex_instance() {
        let schema = get_complex_schema();
        let validator = Validator::new(&schema).unwrap();
        let valid_instance = json!({
          "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
          "username": "testuser",
          "status": "active",
          "profile": { "fullName": "Test User", "age": 30 },
          "tags": ["rust", "validate", "nodejs"],
        });
        let result = perform_validation(&validator, &valid_instance, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fail_on_nested_property_error() {
        let schema = get_complex_schema();
        let validator = Validator::new(&schema).unwrap();
        let invalid_instance = json!({
          "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
          "username": "testuser",
          "status": "active",
          "profile": { "fullName": "Test User", "age": 17 },
          "tags": ["testing"],
        });
        let issues = perform_validation(&validator, &invalid_instance, false).unwrap_err();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "too_small");
        assert_eq!(issues[0].path, "/profile/age");
    }

    #[test]
    fn test_report_multiple_simultaneous_errors() {
        let schema = get_complex_schema();
        let validator = Validator::new(&schema).unwrap();
        let very_invalid_instance = json!({
          "id": "invalid-uuid",
          "username": "a",
          "profile": { "age": 20 },
          "tags": [],
        });
        let issues = perform_validation(&validator, &very_invalid_instance, false).unwrap_err();
        assert_eq!(issues.len(), 5);
        let codes: Vec<_> = issues.iter().map(|issue| &issue.code).collect();
        assert!(codes.contains(&&"pattern_mismatch".to_string()));
        assert!(codes.contains(&&"too_short".to_string()));
        assert!(codes.contains(&&"missing_property".to_string()));
        assert!(codes.contains(&&"too_few_items".to_string()));
    }

    #[test]
    fn test_fail_with_masked_values() {
        let schema = get_complex_schema();
        let validator = Validator::new(&schema).unwrap();
        let invalid_instance = json!({
          "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
          "username": "a",
          "status": "pending",
          "tags": ["masked"],
        });
        let issues = perform_validation(&validator, &invalid_instance, true).unwrap_err();
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert!(!issue.message.contains(r#""a""#));
        assert!(issue.message.contains("value is shorter than 3 characters"));
    }
}