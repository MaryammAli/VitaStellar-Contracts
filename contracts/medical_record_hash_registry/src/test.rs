#[cfg(test)]
mod tests {
    use crate::{Error, MedicalRecordHashRegistry, MedicalRecordHashRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

    fn setup() -> (Env, MedicalRecordHashRegistryClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, MedicalRecordHashRegistry);
        let client = MedicalRecordHashRegistryClient::new(&env, &contract_id);
        (env, client, admin)
    }

    #[test]
    fn test_initialize() {
        let (_env, client, admin) = setup();
        client.initialize(&admin);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (_env, client, admin) = setup();
        client.initialize(&admin);
        let result = client.try_initialize(&admin);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_store_record() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        client.store_record(&admin, &patient_id, &record_hash);
    }

    #[test]
    fn test_duplicate_record_rejected() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        client.store_record(&admin, &patient_id, &record_hash);
        let result = client.try_store_record(&admin, &patient_id, &record_hash);
        assert_eq!(result, Err(Ok(Error::DuplicateRecord)));
    }

    #[test]
    fn test_verify_record() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        client.store_record(&admin, &patient_id, &record_hash);
        let result = client.verify_record(&patient_id, &record_hash);
        assert!(result);
    }

    #[test]
    fn test_verify_nonexistent_record() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        let result = client.verify_record(&patient_id, &record_hash);
        assert!(!result);
    }

    #[test]
    fn test_multiple_records_same_patient() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let hash1: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        let hash2: BytesN<32> = BytesN::from_array(&env, &[2u8; 32]);
        client.store_record(&admin, &patient_id, &hash1);
        client.store_record(&admin, &patient_id, &hash2);
        assert!(client.verify_record(&patient_id, &hash1));
        assert!(client.verify_record(&patient_id, &hash2));
        let count = client.get_record_count(&patient_id);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_patient_by_hash() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        client.store_record(&admin, &patient_id, &record_hash);
        let result = client.get_patient_by_hash(&record_hash);
        assert_eq!(result, Some(patient_id));
    }

    #[test]
    fn test_get_patient_records() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let hash1: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        let hash2: BytesN<32> = BytesN::from_array(&env, &[2u8; 32]);
        client.store_record(&admin, &patient_id, &hash1);
        client.store_record(&admin, &patient_id, &hash2);
        let records = client.get_patient_records(&patient_id);
        assert!(records.is_some());
        assert_eq!(records.unwrap().record_count, 2);
    }

    #[test]
    fn test_immutability() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        client.store_record(&admin, &patient_id, &record_hash);
        let result = client.try_store_record(&admin, &patient_id, &record_hash);
        assert_eq!(result, Err(Ok(Error::DuplicateRecord)));
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::Unauthorized as u32, 100);
        assert_eq!(Error::InvalidId as u32, 206);
        assert_eq!(Error::InvalidSignature as u32, 207);
        assert_eq!(Error::InvalidRecordHash as u32, 251);
        assert_eq!(Error::NotInitialized as u32, 300);
        assert_eq!(Error::AlreadyInitialized as u32, 301);
        assert_eq!(Error::ContractPaused as u32, 302);
        assert_eq!(Error::DeadlineExceeded as u32, 306);
        assert_eq!(Error::DuplicateRecord as u32, 402);
        assert_eq!(Error::RecordNotFound as u32, 403);
        assert_eq!(Error::InsufficientFunds as u32, 500);
        assert_eq!(Error::StorageFull as u32, 502);
        assert_eq!(Error::CrossChainTimeout as u32, 702);
    }

    #[test]
    fn test_get_suggestion_returns_expected_hint() {
        use crate::errors::get_suggestion;
        use soroban_sdk::symbol_short;
        assert_eq!(
            get_suggestion(Error::Unauthorized),
            symbol_short!("CHK_AUTH")
        );
        assert_eq!(
            get_suggestion(Error::NotInitialized),
            symbol_short!("INIT_CTR")
        );
        assert_eq!(
            get_suggestion(Error::AlreadyInitialized),
            symbol_short!("ALREADY")
        );
        assert_eq!(
            get_suggestion(Error::RecordNotFound),
            symbol_short!("CHK_ID")
        );
        assert_eq!(
            get_suggestion(Error::InsufficientFunds),
            symbol_short!("ADD_FUND")
        );
        assert_eq!(get_suggestion(Error::StorageFull), symbol_short!("CLN_OLD"));
    }

    #[test]
    fn test_pause_unpause() {
        let (env, client, admin) = setup();
        client.initialize(&admin);

        // Initially not paused
        assert!(!client.is_paused());

        // Pause
        client.pause(&admin);
        assert!(client.is_paused());

        // Unpause
        client.unpause(&admin);
        assert!(!client.is_paused());
    }

    #[test]
    fn test_store_record_rejected_when_paused() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);

        client.pause(&admin);

        let result = client.try_store_record(&admin, &patient_id, &record_hash);
        assert_eq!(result, Err(Ok(Error::ContractPaused)));
    }

    #[test]
    fn test_store_record_works_after_unpause() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_id = Address::generate(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);

        client.pause(&admin);
        client.unpause(&admin);

        // Should work now
        client.store_record(&admin, &patient_id, &record_hash);
        assert!(client.verify_record(&patient_id, &record_hash));
    }

    #[test]
    fn test_pause_requires_admin() {
        let (env, client, _admin) = setup();
        client.initialize(&_admin);
        let non_admin = Address::generate(&env);

        let result = client.try_pause(&non_admin);
        assert_eq!(result, Err(Ok(Error::Unauthorized)));
    }

    #[test]
    fn test_health_check_returns_ok() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        env.ledger().set_timestamp(10_000);

        let (status, version, timestamp) = client.health_check();
        assert_eq!(status, soroban_sdk::symbol_short!("OK"));
        assert_eq!(version, 1);
        assert_eq!(timestamp, 10_000);
    }

    #[test]
    fn test_health_check_not_init() {
        let (env, client, _admin) = setup();
        env.ledger().set_timestamp(10_000);

        let (status, _version, _timestamp) = client.health_check();
        assert_eq!(status, soroban_sdk::symbol_short!("NOT_INIT"));
    }

    #[test]
    fn test_health_check_paused() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        client.pause(&admin);
        env.ledger().set_timestamp(10_000);

        let (status, _version, _timestamp) = client.health_check();
        assert_eq!(status, soroban_sdk::symbol_short!("PAUSED"));
    }
}
