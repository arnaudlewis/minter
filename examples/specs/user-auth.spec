spec user-auth v1.0.0
title "User Authentication"

description
  Handles user registration and login with token-based authentication.

motivation
  Every action in the system requires an authenticated user context.

behavior register-user [happy_path]
  "A new user can register with a valid email and password"

  nfr
    security#password-hashing

  given
    No account exists for the provided email

  when register
    email = "alice@example.com"
    password = "s3cure-p4ss!"

  then returns registration-result
    assert user_id is_present
    assert status == "created"

behavior login-user [happy_path]
  "A registered user can log in and receive an access token"

  given
    @user = User { email: "alice@example.com", password_hash: "..." }

  when login
    email = "alice@example.com"
    password = "s3cure-p4ss!"

  then returns auth-token
    assert token is_present
    assert expires_in > 0

behavior register-duplicate-email [error_case]
  "Registration fails when the email is already taken"

  given
    @existing = User { email: "alice@example.com" }

  when register
    email = "alice@example.com"
    password = "n3w-p4ss!"

  then returns error
    assert status == "conflict"
    assert message == "email already registered"

behavior login-wrong-password [error_case]
  "Login fails with invalid credentials"

  nfr
    security#brute-force-protection

  given
    @user = User { email: "alice@example.com", password_hash: "..." }

  when login
    email = "alice@example.com"
    password = "wrong-password"

  then returns error
    assert status == "unauthorized"
    assert message == "invalid credentials"
