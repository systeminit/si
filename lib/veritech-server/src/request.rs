use si_crypto::VeritechDecryptionKey;
use si_pool_noodle::{
    ActionRunRequest,
    BeforeFunction,
    DebugRequest,
    ManagementRequest,
    ResolverFunctionRequest,
    SchemaVariantDefinitionRequest,
    SensitiveStrings,
    ValidationRequest,
};
use veritech_core::{
    VeritechValueDecryptError,
    decrypt_value_tree,
};

pub trait DecryptRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut SensitiveStrings,
        decryption_key: &VeritechDecryptionKey,
    ) -> Result<(), VeritechValueDecryptError>;
}

impl DecryptRequest for ResolverFunctionRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut SensitiveStrings,
        decryption_key: &VeritechDecryptionKey,
    ) -> Result<(), VeritechValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

impl DecryptRequest for ActionRunRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut SensitiveStrings,
        decryption_key: &VeritechDecryptionKey,
    ) -> Result<(), VeritechValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

impl DecryptRequest for ValidationRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut SensitiveStrings,
        decryption_key: &VeritechDecryptionKey,
    ) -> Result<(), VeritechValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

impl DecryptRequest for SchemaVariantDefinitionRequest {
    fn decrypt(
        &mut self,
        _sensitive_strings: &mut SensitiveStrings,
        _decryption_key: &VeritechDecryptionKey,
    ) -> Result<(), VeritechValueDecryptError> {
        // No before funcs defined!
        Ok(())
    }
}

impl DecryptRequest for ManagementRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut SensitiveStrings,
        decryption_key: &VeritechDecryptionKey,
    ) -> Result<(), VeritechValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

impl DecryptRequest for DebugRequest {
    fn decrypt(
        &mut self,
        sensitive_strings: &mut SensitiveStrings,
        decryption_key: &VeritechDecryptionKey,
    ) -> Result<(), VeritechValueDecryptError> {
        decrypt_before_func_args(&mut self.before, sensitive_strings, decryption_key)
    }
}

fn decrypt_before_func_args(
    before: &mut Vec<BeforeFunction>,
    sensitive_strings: &mut SensitiveStrings,
    decryption_key: &VeritechDecryptionKey,
) -> Result<(), VeritechValueDecryptError> {
    for func in before {
        decrypt_value_tree(&mut func.arg, sensitive_strings, decryption_key)?;
    }

    Ok(())
}
