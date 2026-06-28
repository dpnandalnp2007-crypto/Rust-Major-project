use iam_platform::{
    models::PaginationQuery,
    utils::{
        api_key_gen::{generate_api_key, hash_api_key},
        jwt::{create_access_token, create_refresh_token, verify_access_token, verify_refresh_token},
        password::{hash_password, verify_password},
    },
};
use uuid::Uuid;

#[test]
fn test_password_hashing_and_verification() {
    let raw_pwd = "my_secure_password_123";
    let hash = hash_password(raw_pwd).expect("Failed to hash password");
    
    assert_ne!(raw_pwd, hash);
    assert!(verify_password(raw_pwd, &hash).expect("Verification error"));
    assert!(!verify_password("wrong_password", &hash).expect("Verification error"));
}

#[test]
fn test_jwt_access_token_lifecycle() {
    let user_id = Uuid::new_v4();
    let email = "alice@acme.com";
    let secret = "super_secret_test_key_for_jwt_verification";
    
    let token = create_access_token(user_id, email, secret, 15).expect("Failed to generate access token");
    let claims = verify_access_token(&token, secret).expect("Failed to verify access token");
    
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.email, email);
    assert_eq!(claims.token_type, "access");
}

#[test]
fn test_jwt_refresh_token_lifecycle() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let secret = "super_secret_test_key_for_jwt_verification";
    
    let token = create_refresh_token(user_id, session_id, secret, 7).expect("Failed to generate refresh token");
    let claims = verify_refresh_token(&token, secret).expect("Failed to verify refresh token");
    
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.session_id, session_id);
    assert_eq!(claims.token_type, "refresh");
}

#[test]
fn test_jwt_invalid_secret_fails() {
    let user_id = Uuid::new_v4();
    let token = create_access_token(user_id, "test@test.com", "secret_1", 15).unwrap();
    assert!(verify_access_token(&token, "wrong_secret").is_err());
}

#[test]
fn test_api_key_generation_and_hashing() {
    let (raw_key, prefix, hash) = generate_api_key();
    
    assert!(raw_key.starts_with("iam_live_"));
    assert_eq!(prefix.len(), 16);
    assert!(raw_key.starts_with(&prefix));
    
    let rehashed = hash_api_key(&raw_key);
    assert_eq!(hash, rehashed);
}

#[test]
fn test_pagination_offset_and_limits() {
    let query = PaginationQuery {
        page: Some(3),
        limit: Some(25),
        search: None,
    };
    
    assert_eq!(query.limit(), 25);
    assert_eq!(query.offset(), 50); // (3 - 1) * 25
    
    let default_query = PaginationQuery {
        page: None,
        limit: None,
        search: None,
    };
    assert_eq!(default_query.limit(), 20);
    assert_eq!(default_query.offset(), 0);
}
