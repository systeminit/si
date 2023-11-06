use cyclone_core::{
    decrypt_value_tree, ActionRunRequest, BeforeFunction, CycloneDecryptionKey,
    CycloneSensitiveStrings, CycloneValueDecryptError, ReconciliationRequest,
    ResolverFunctionRequest, SchemaVariantDefinitionRequest, ValidationRequest,
};

pub trait DecryptRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut CycloneSensitiveStrings,
        decryption_key: &CycloneDecryptionKey,
    ) -> Result<(), CycloneValueDecryptError>;
}

impl DecryptRequest for ResolverFunctionRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut CycloneSensitiveStrings,
        decryption_key: &CycloneDecryptionKey,
    ) -> Result<(), CycloneValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

impl DecryptRequest for ActionRunRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut CycloneSensitiveStrings,
        decryption_key: &CycloneDecryptionKey,
    ) -> Result<(), CycloneValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

impl DecryptRequest for ReconciliationRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut CycloneSensitiveStrings,
        decryption_key: &CycloneDecryptionKey,
    ) -> Result<(), CycloneValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

impl DecryptRequest for ValidationRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut CycloneSensitiveStrings,
        decryption_key: &CycloneDecryptionKey,
    ) -> Result<(), CycloneValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

impl DecryptRequest for SchemaVariantDefinitionRequest {
    fn decrypt(
        &mut self,
        _sensitive_strings: &mut CycloneSensitiveStrings,
        _decryption_key: &CycloneDecryptionKey,
    ) -> Result<(), CycloneValueDecryptError> {
        // No before funcs defined!
        Ok(())
    }
}

fn decrypt_before_func_args(
    before: &mut Vec<BeforeFunction>,
    sensitive_strings: &mut CycloneSensitiveStrings,
    decryption_key: &CycloneDecryptionKey,
) -> Result<(), CycloneValueDecryptError> {
    for func in before {
        decrypt_value_tree(&mut func.arg, sensitive_strings, decryption_key)?;
    }

    Ok(())
}
