---
# JeebsAI API Documentation

## PGP Authentication & Chat Endpoints

### 1. Register (PGP)
- **Endpoint:** `POST /api/register`
- **Body:**
  ```json
  {
    "username": "string (3-32 chars, a-zA-Z0-9_-)" ,
    "email": "string (optional)",
    "pgp_public_key": "string (ASCII-armored PGP public key)"
  }
  ```
- **Response:**
  - `201 Created` on success:
    ```json
    { "status": "registered", "username": "..." }
    ```
  - `400/409` on error with `error` field.

### 2. Login (PGP)
- **Endpoint:** `POST /api/login_pgp`
- **Body:**
  ```json
  {
    "username": "string",
    "signed_message": "string (PGP clearsigned message)",
    "remember_me": "boolean (optional)"
  }
  ```
- **Response:**
  - `200 OK` on success:
    ```json
    {
      "status": "success",
      "username": "...",
      "is_admin": true,
      "is_trainer": false,
      "token": "JWT or session token"
    }
    ```
  - `400/401` on error with `error` field.

- **Challenge Format:**
  The message to sign must be:
  ```
  LOGIN:username:unix_timestamp
  ```
  (timestamp must be within 5 minutes of server time)

### 3. Auth Status
- **Endpoint:** `GET /api/auth/status`
- **Response:**
  ```json
  {
    "logged_in": true,
    "username": "string|null",
    "is_admin": true,
    "is_trainer": false,
    "token": "string|null"
  }
  ```

### 4. Logout
- **Endpoint:** `POST /api/logout`
- **Response:**
  - `200 OK` on success.

### 5. Chat (Authenticated)
- **Endpoint:** `POST /api/chat`
- **Headers:**
  - `Authorization: Bearer <token>` (or session cookie)
- **Body:**
  ```json
  { "message": "string" }
  ```
- **Response:**
  ```json
  {
    "response": "Jeebs response",
    "username": "...",
    "is_admin": true
  }
  ```
  - `401` if not authenticated.

### 6. Chat History (Optional)
- **Endpoint:** `GET /api/chat/history?limit=50`
- **Response:**
  ```json
  [
    { "id": 1, "role": "user|jeebs", "message": "..." }
  ]
  ```

---