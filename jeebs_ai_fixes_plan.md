# Implementation Plan: JeebsAI Fixes (Brain Graph, Proposals, Session)

## Objective
Address three reported issues with the JeebsAI dashboard: missing data in the Brain Graph visualization, failed API calls when approving evolution proposals, and a 404 error on the Session Debug endpoint.

## Issues and Solutions

### 1. Brain Graph Visualization Empty ("No Nodes")
**Problem:** The `visualize.html` frontend queries `/api/brain/visualize`. The backend endpoint in `brain_parsing_api.rs` uses `build_knowledge_graph()`, which only queries the `brain_nodes` table. However, Jeebs now heavily stores conversational facts inside `jeebs_store` under the `chat:fact:%` keys.
**Solution:** (Already Implemented)
- Modified `build_knowledge_graph` in `src/brain_parser.rs` to fetch records matching `chat:fact:%` from `jeebs_store`.
- Deserialized the stored facts, extracted their `canonical` and `fact` values, and fed them through the existing text NLP/regex `BrainParser`.
- Sent these along with the regular node structures to generate valid nodes and edges for the UI.

### 2. Cannot Accept Evolution Proposals
**Problem:** A strict mismatch between frontend API calls and the actix-web backend definitions. `evolution.html` tries to hit `/api/admin/evolution/update/{id}/apply`, but `evolution.rs` correctly defines the endpoints as `/api/admin/evolution/apply/{id}` and `/api/admin/evolution/deny/{id}`.
**Solution:** (Already Implemented)
- Fixed the JS fetch methods `applyUpdate` and `denyUpdate` inside `webui/evolution.html` to align with the backend router map. 

### 3. Session Debug Status 404
**Problem:** The `status.html` page polls `/api/auth/session` to display robust debugging identity information. A `404 Not Found` indicates the endpoint exists in code but fell out of the central router. Checking `src/auth/mod.rs` reveals the function `auth_session` stands at line 812, wrapped with `#[get("/api/auth/session")]`. However, it lacks registration in `src/main.rs`.
**Solution:** (Pending)
- Add `.service(auth::auth_session)` to the application factory in `src/main.rs` directly succeeding the other auth endpoints.
- Compile and restart the service via `bash scripts/vps_deploy.sh`.

## Review Request
Please review this implementation plan. I will immediately register the final missing API endpoint and reboot your VPS instance once you approve.
