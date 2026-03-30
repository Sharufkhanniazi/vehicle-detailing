mod common;
use common::test_db::TestDb;
use auth_service::models::{LoginUser, RegisterUser};
use auth_service::services::auth::AuthService;
use shared_auth::models::UserRole;

#[tokio::test]
async fn test_user_registration_success() {
   
    let test_db = TestDb::new().await;

    let auth_service = AuthService::new(test_db.pool.clone());

    // test data
    let register_user = RegisterUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        user_role: UserRole::CUSTOMER,
    };

    let result = auth_service.register(register_user).await;

    assert!(result.is_ok(), "Registration failed: {:?}", result.err());

    let response = result.unwrap();

    assert!(response.contains("Registration successful"));
    assert!(response.contains("test@example.com"));

    // verify user was created in database
    let user = sqlx::query!("SELECT email, is_email_verified FROM users WHERE email = $1", "test@example.com")
        .fetch_one(&test_db.pool)
        .await
        .expect("User should exist");

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.is_email_verified, false);

    // verify customer profile was created
    let profile = sqlx::query!(
        "SELECT user_id FROM customer_profiles WHERE user_id = (SELECT id FROM users WHERE email = $1)",
        "test@example.com"
    )
    .fetch_optional(&test_db.pool)
    .await
    .expect("Query should succeed");

   
    assert!(profile.is_some(), "Customer profile should exist");

    test_db.cleanup().await;
}

 #[tokio::test]
async fn test_user_login_success() {

    let test_db = TestDb::new().await;

    let auth_service = AuthService::new(test_db.pool.clone());

    // register user
    let register_user = RegisterUser {
        username: "loginuser".to_string(),
        email: "login@example.com".to_string(),
        password: "securepass123".to_string(),
        user_role: UserRole::CUSTOMER,
    };

    let register_result = auth_service.register(register_user).await;
    
    register_result.unwrap();

    // manually verify email
    let user = sqlx::query!("SELECT id FROM users WHERE email = $1", "login@example.com")
        .fetch_one(&test_db.pool)
        .await
        .expect("User should exist");

    sqlx::query!("UPDATE users SET is_email_verified = true WHERE id = $1", user.id)
        .execute(&test_db.pool)
        .await
        .expect("Failed to verify email");

    // test login
    let login_user = LoginUser {
        username: "loginuser".to_string(),
        password: "securepass123".to_string(),
    };

    let result = auth_service.login(login_user).await;

    assert!(result.is_ok(), "Login failed: {:?}", result.err());

    let login_response = result.unwrap();

    assert_eq!(login_response.user.username, "loginuser");
    assert_eq!(login_response.user.email, "login@example.com");
    assert!(!login_response.token.is_empty());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_login_with_unverified_email_fails() {

    let test_db = TestDb::new().await;

    let auth_service = AuthService::new(test_db.pool.clone());

    // register user without verifying email
    let register_user = RegisterUser {
        username: "unverified".to_string(),
        email: "unverified@example.com".to_string(),
        password: "password123".to_string(),
        user_role: UserRole::CUSTOMER,
    };

    let register_result = auth_service.register(register_user).await;
    
    register_result.unwrap();

    // attempt login
    let login_user = LoginUser {
        username: "unverified".to_string(),
        password: "password123".to_string(),
    };

    let result = auth_service.login(login_user).await;

    assert!(result.is_err(), "Login should have failed but succeeded");

    test_db.cleanup().await;
}