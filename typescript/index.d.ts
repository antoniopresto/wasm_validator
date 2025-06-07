/**
 * A comprehensive list of stable, machine-readable error codes that can be returned
 * by the validator. This allows for robust, programmatic handling of different
 * validation failures.
 */
export type ValidationErrorCode =
  | 'invalid_schema'
  | 'additional_items'
  | 'additional_properties'
  | 'any_of_mismatch'
  | 'regex_backtrack_limit'
  | 'const_mismatch'
  | 'no_match_in_contains'
  | 'invalid_content_encoding'
  | 'invalid_media_type'
  | 'custom_error'
  | 'enum_mismatch'
  | 'exclusive_max'
  | 'exclusive_min'
  | 'disallowed_value'
  | 'format_mismatch'
  | 'invalid_utf8'
  | 'too_large'
  | 'too_many_items'
  | 'too_long'
  | 'too_many_properties'
  | 'too_small'
  | 'too_few_items'
  | 'too_short'
  | 'too_few_properties'
  | 'not_a_multiple'
  | 'negated_schema_match'
  | 'one_of_multiple_matches'
  | 'one_of_no_match'
  | 'pattern_mismatch'
  | 'invalid_property_name'
  | 'missing_property'
  | 'invalid_type'
  | 'unevaluated_items'
  | 'unevaluated_properties'
  | 'duplicate_items'
  | 'schema_reference_error';

/**
 * Describes the structure of a single validation error.
 * When validation fails, an array of these objects is thrown.
 */
export interface ValidationIssue {
  /**
   * A JSON Pointer (RFC 6901) to the location in the instance where the error occurred.
   * An empty string indicates the root of the instance.
   * @example "/profile/age"
   */
  path: string;
  /**
   * A human-readable description of the validation error. If `mask_values` was set
   * to true during validation, this message will be redacted to avoid exposing
   * potentially sensitive data.
   * @example "17 is less than the minimum of 18"
   * @example "value is shorter than 3 characters"
   */
  message: string;
  /**
   * A stable, machine-readable code representing the specific type of validation error.
   * This is useful for programmatic error handling.
   */
  code: ValidationErrorCode;
}

/**
 * A generic type representing a JSON Schema document.
 */
export type JSONSchema = Record<string, any>;

/**
 * The main validation function exported from the WebAssembly module.
 * It validates a JSON object (`instance`) against a JSON Schema (`schema`).
 *
 * If validation is successful, the function returns normally (void).
 * If validation fails, the function throws an array of `ValidationIssue` objects.
 *
 * @param schema The JSON Schema object to validate against.
 * @param instance The JSON object instance to be validated.
 * @param mask_values If true, potentially sensitive data in the instance will be
 * redacted from the error messages. Defaults to `false`.
 * @throws {ValidationIssue[]} An array of validation issues if the instance is invalid.
 * @example
 * ```javascript
 * import { validate } from 'wasm-validator';
 *
 * const schema = { type: 'string', minLength: 5 };
 * const instance = "test";
 *
 * try {
 * validate(schema, instance);
 * console.log("Validation successful!");
 * } catch (errors) {
 * console.error("Validation failed:", errors);
 * // errors is an array of ValidationIssue objects
 * // e.g., [{ path: '', message: '"test" is shorter than 5 characters', code: 'too_short' }]
 * }
 * ```
 */
export function validate<T extends any>(
  schema: JSONSchema,
  instance: T,
  mask_values?: boolean,
): T;

export class ValidationError {
  issues: ValidationIssue[]
}

export function isValidationError(
  value: any,
): value is ValidationError;

export function assertValidationError(
  value: any,
): asserts value is ValidationError;


