const wv = require('./wasm_validator.js');

module.exports = {}

class ValidationError extends Error {
  issues = []
}

function isValidationError(value){
  return value instanceof ValidationError
}

function assertValidationError(value){
 if(!isValidationError(value)){
   throw new Error(`Expected value to be a ValidationError`)
 }
}

module.exports.isValidationError = isValidationError;
module.exports.assertValidationError = assertValidationError;
module.exports.ValidationError = ValidationError;

module.exports.validate = function validate(schema, instance, mask_values){
  try {
     wv.validate(instance, schema, mask_values);
     return instance
  }catch (e) {
    if(!Array.isArray(e)) throw e
    const error = new ValidationError()
    error.issues = e
    throw error
  }
}