# Fix Verification - Commands to Run

## Step 1: Verify Compilation

```bash
# Clean build to ensure no stale artifacts
cargo clean

# Check for errors
cargo check

# Build release
cargo build --release

# If all succeed ✅, proceed to Step 2
```

## Step 2: Run the Server

```bash
# Start the server
cargo run

# Or if you have a built binary
./target/release/jeebs

# Server should start without errors
# Look for messages like:
# "Starting server on 0.0.0.0:8080"
```

## Step 3: Test Chat Endpoint (Fixed)

### Option A: With Session Cookie (Recommended)

```bash
# First, register an account
curl -X POST http://localhost:8080/api/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass123"}'

# Then login
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"username":"testuser","password":"testpass123"}'

# Now test chat with the session cookie
curl -X POST http://localhost:8080/api/jeebs \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{"prompt":"Hello, how are you?"}'

# Expected Response (200 OK):
# {"response":"I'm doing well..."}
```

### Option B: With JWT Token

```bash
# After logging in, get the token from login response
# Then use it:

curl -X POST http://localhost:8080/api/jeebs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN_HERE" \
  -d '{"prompt":"Hello"}'

# Expected Response (200 OK):
# {"response":"..."}
```

## Step 4: Test New Deep Learning Endpoints

### 1. Start a Learning Session

```bash
curl -X POST http://localhost:8080/api/learning/start-deep-learning \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"topic":"Rust Programming"}'

# Expected Response (200 OK):
# {
#   "success": true,
#   "session_id": "uuid-here",
#   "topic": "Rust Programming",
#   "subtopics": [...]
# }
```

### 2. Add a Learned Fact

```bash
curl -X POST http://localhost:8080/api/learning/add-fact \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "session_id": "YOUR_SESSION_ID",
    "fact": "Rust prevents null pointer exceptions",
    "source": "The Rust Book",
    "importance": 0.95
  }'

# Expected Response (200 OK):
# {"success": true, "message": "Fact learned and stored"}
```

### 3. Check Learning Statistics

```bash
curl http://localhost:8080/api/learning/statistics \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Expected Response (200 OK):
# {
#   "success": true,
#   "statistics": {
#     "total_learning_sessions": 1,
#     "total_study_hours": 0.5,
#     "total_facts_learned": 1,
#     ...
#   }
# }
```

## Step 5: Verify No 405 Errors

All of the above commands should return:
- ✅ 200 OK
- ✅ Valid JSON response
- ❌ NOT 405 Method Not Allowed
- ❌ NOT 404 Not Found

## Troubleshooting

### Still Getting 405 Errors?

1. **Check compilation**:
   ```bash
   cargo check
   # Look for any errors
   ```

2. **Check server started**:
   - Server should print startup messages
   - Port 8080 should be listening: `lsof -i :8080`

3. **Check authentication**:
   - Ensure you're using JWT token or session cookie
   - Chat endpoint requires auth

4. **Check request format**:
   - Use POST, not GET for `/api/jeebs`
   - Include `Content-Type: application/json` header
   - Include valid JSON body

### Build Errors?

1. **Check Cargo.toml** has all dependencies:
   ```bash
   grep -E "uuid|chrono|serde" Cargo.toml
   ```

2. **Try clean rebuild**:
   ```bash
   cargo clean
   cargo build --release
   ```

## Success Indicators

✅ **Compilation succeeds** - No cargo errors
✅ **Server starts** - Sees startup messages
✅ **Chat returns 200** - Not 405
✅ **New endpoints work** - Deep learning endpoints respond
✅ **Session works** - Can login and chat in same session

## Summary

All fixes have been applied to:
- ✅ `src/main.rs` - Removed duplicate registration
- ✅ `src/cortex.rs` - Reorganized struct definitions
- ✅ `src/deep_learning.rs` - Added UUID import
- ✅ `src/lib.rs` - Modules properly exported
- ✅ `src/main.rs` - All endpoints registered

**The chat 405 error should now be completely fixed.** 🎉
