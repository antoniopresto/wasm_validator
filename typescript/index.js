const {
  WasmValidator: WasmValidatorRaw,
  validate: validateRaw,
} = require('./wasm_validator.js');

class ValidationError extends Error {
  constructor(issues) {
    const issue_list = issues.map(i => `- ${i.path || 'instance'}: ${i.message}`).join('\n');
    super(`Validation failed with ${issues.length} error(s):\n${issue_list}`);
    this.name = 'ValidationError';
    this.issues = issues;
  }
}

function isValidationError(value) {
  return value instanceof ValidationError;
}

function assertValidationError(value) {
  if (!isValidationError(value)) {
    throw new Error(`Expected value to be a ValidationError`);
  }
}

// The new performant, stateful validator class.
class WasmValidator {
  #validator; // Private field for the raw Wasm instance

  constructor(schema, mask_values = false) {
    try {
      this.#validator = new WasmValidatorRaw(schema, mask_values);
    } catch(e) {
      if(!Array.isArray(e)) throw e
      throw new ValidationError(e)
    }
  }

  validate(instance) {
    try {
      this.#validator.validate(instance);
      return instance;
    } catch (e) {
      if (!Array.isArray(e)) throw e;
      throw new ValidationError(e);
    }
  }
}

// The original stateless function, useful for one-off validations.
function validate(schema, instance, mask_values) {
  try {
    validateRaw(schema, instance, mask_values);
    return instance;
  } catch (e) {
    if (!Array.isArray(e)) throw e;
    throw new ValidationError(e);
  }
}

module.exports.validate = validate;
module.exports.WasmValidator = WasmValidator;
module.exports.ValidationError = ValidationError;
module.exports.isValidationError = isValidationError;
module.exports.assertValidationError = assertValidationError;