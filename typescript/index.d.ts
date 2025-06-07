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
 */
export interface ValidationIssue {
  /**
   * A JSON Pointer (RFC 6901) to the location in the instance where the error occurred.
   * @example "/profile/age"
   */
  path: string;
  /**
   * A human-readable description of the validation error. This message may be
   * redacted if `mask_values` was set to true.
   * @example "17 is less than the minimum of 18"
   */
  message: string;
  /**
   * A stable, machine-readable code representing the specific type of validation error.
   */
  code: ValidationErrorCode;
}

/**
 * A generic type representing a JSON Schema document.
 */
export type JSONSchema = Record<string, any>;

/**
 * An Error subclass that is thrown when validation fails.
 * It contains an array of `ValidationIssue` objects.
 */
export class ValidationError extends Error {
  /** An array of detailed validation issues. */
  issues: ValidationIssue[];
  constructor(issues: ValidationIssue[]);
}

/**
 * A stateful, performant validator that pre-compiles a JSON schema.
 * Use this class when you need to validate multiple instances against the same schema.
 */
export class WasmValidator {
  /**
   * Creates and compiles a new validator instance.
   * @param schema The JSON Schema object to validate against.
   * @param mask_values If true, potentially sensitive data will be redacted from error messages. Defaults to `false`.
   * @throws {ValidationError} Throws if the schema itself is invalid.
   */
  constructor(schema: JSONSchema, mask_values?: boolean);

  /**
   * Validates a JSON object instance against the pre-compiled schema.
   * @param instance The JSON instance to validate.
   * @returns The validated instance if validation is successful.
   * @throws {ValidationError} Throws if the instance is invalid.
   */
  validate<T extends any>(instance: T): T;
}

/**
 * A stateless validation function for one-off use.
 * This is less performant for repeated use as it compiles the schema on every call.
 *
 * @param schema The JSON Schema object to validate against.
 * @param instance The JSON instance to validate.
 * @param mask_values If true, potentially sensitive data will be redacted from error messages. Defaults to `false`.
 * @returns The validated instance if validation is successful.
 * @throws {ValidationError} Throws if the instance is invalid.
 */
export function validate<T extends any>(
    schema: JSONSchema,
    instance: T,
    mask_values?: boolean
): T;

/**
 * Type guard to check if a value is a `ValidationError`.
 * @param value The value to check.
 * @returns True if the value is a `ValidationError`.
 */
export function isValidationError(value: any): value is ValidationError;

/**
 * Asserts that a value is a `ValidationError`, throwing an error otherwise.
 * @param value The value to check.
 */
export function assertValidationError(value: any): asserts value is ValidationError;