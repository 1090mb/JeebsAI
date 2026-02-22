# Brain Parser - Deployment Guide

## Pre-Deployment Checklist

### Code Review
- [x] All modules compile without errors
- [x] No unsafe code blocks
- [x] Proper error handling throughout
- [x] Memory safety verified
- [x] Dependencies checked

### Testing
```bash
# Run all tests
cargo test

# Run parser-specific tests
cargo test brain_parser

# Check for warnings
cargo check --all-targets
```

### Build Verification
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check binaries exist
ls -lh target/release/jeebs
```

## Deployment Steps

### 1. Local Testing
```bash
# Start the server
cargo run

# Test parsing endpoint
curl -X POST http://localhost:8080/api/brain/parse \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "test_1",
    "key": "Test Entity",
    "value": "This is a test brain node for verification"
  }'

# Test graph building (if you have data)
curl -X POST http://localhost:8080/api/brain/graph/build

# Test statistics
curl http://localhost:8080/api/brain/graph/statistics
```

### 2. Staging Deployment
```bash
# Build release binary
cargo build --release

# Test on staging server
scp target/release/jeebs user@staging:/opt/jeebs/

# Restart service
ssh user@staging "sudo systemctl restart jeebs"

# Verify endpoints are responding
curl https://staging.jeebs.ai/api/brain/parse -X POST ...
```

### 3. Production Deployment
```bash
# Create backup
pg_dump jeebs_prod > backup_$(date +%Y%m%d).sql

# Build release
cargo build --release

# Deploy binary
scp target/release/jeebs user@prod:/opt/jeebs/

# Restart service
ssh user@prod "sudo systemctl restart jeebs"

# Health check
curl https://jeebs.ai/api/brain/graph/statistics

# Monitor logs
ssh user@prod "tail -f /var/log/jeebs/jeebs.log"
```

## Database Considerations

The brain parser works with existing brain tables:
- `brain` table (key, value, label, summary, created_at)
- `triples` table (subject, predicate, object, confidence)

No migration needed - fully backward compatible.

## Configuration

### Environment Variables
```bash
# Existing variables remain the same
DATABASE_URL=sqlite:./jeebs.db
PORT=8080

# No new variables required
```

### Performance Tuning
```sql
-- Optional: Add indexes for faster queries
CREATE INDEX idx_brain_key ON brain(key);
CREATE INDEX idx_brain_value ON brain(value);
CREATE INDEX idx_brain_label ON brain(label);
```

## Monitoring

### Key Metrics to Track
1. **Parser Performance**
   - Graph build time
   - Entities extracted per node
   - Confidence scores distribution

2. **API Endpoints**
   - Response times
   - Error rates
   - Request volume

3. **Graph Statistics**
   - Total entities
   - Total relationships
   - Graph size (memory usage)

### Logging
The system logs to:
- `jeebs.log` - Main application log
- Standard output during development

## Rollback Plan

If issues occur:

### Step 1: Identify Issue
```bash
# Check recent logs
tail -100 /var/log/jeebs/jeebs.log

# Look for parser errors
grep -i "parser\|brain_parsing" jeebs.log
```

### Step 2: Rollback Binary
```bash
# Stop service
sudo systemctl stop jeebs

# Restore previous binary
sudo cp /opt/jeebs/jeebs.backup /opt/jeebs/jeebs

# Start service
sudo systemctl start jeebs

# Verify
curl http://localhost:8080/api/brain/parse -X POST ...
```

### Step 3: Verify Functionality
```bash
# Test critical endpoints
curl http://localhost:8080/api/chat
curl http://localhost:8080/api/brain/parse -X POST -d '{...}'
```

## Performance Expectations

### Single Node Parsing
- Time: 5-50ms (depending on text length)
- Memory: < 1MB per node

### Graph Building (1000 nodes)
- Time: 500-2000ms
- Memory: 50-200MB
- Entities: typically 5-15 per node

### Query Operations
- Entity lookup: < 5ms
- Category lookup: < 5ms
- Statistics: 10-50ms

## Scaling Considerations

### For Large Brain (10,000+ nodes)
1. Cache graph for 5-10 minute intervals
2. Implement pagination for entity/category queries
3. Consider partitioning by category
4. Monitor memory usage

### Optimization Options
```rust
// Cache graph in memory
let cached_graph = Arc::new(
    build_knowledge_graph(&db, &parser).await?
);

// Reuse cache for multiple queries
let results = cached_graph.query_by_entity("Rust");
```

## Troubleshooting

### Issue: Slow Graph Building
**Solution:**
- Check database query performance
- Verify database indexes exist
- Cache results in memory
- Run during off-peak hours

### Issue: High Memory Usage
**Solution:**
- Reduce cache size or duration
- Process nodes in batches
- Implement result pagination

### Issue: Incorrect Entity Detection
**Solution:**
- Review pattern definitions
- Adjust confidence thresholds
- Add custom patterns for domain-specific entities

### Issue: API Timeout
**Solution:**
- Increase timeout in Actix config
- Build and cache graphs proactively
- Implement async processing

## Documentation Links
- Full Framework: `BRAIN_PARSING_FRAMEWORK.md`
- Integration Guide: `BRAIN_PARSER_INTEGRATION.md`
- Examples: `BRAIN_PARSER_EXAMPLES.rs`
- Summary: `BRAIN_PARSER_IMPLEMENTATION_SUMMARY.md`

## Support & Maintenance

### Regular Maintenance
- Monitor parser accuracy
- Update pattern definitions quarterly
- Gather entity/relationship statistics
- Plan ML-based improvements

### Issue Reporting
Document issues with:
1. Input data (sample node)
2. Expected output
3. Actual output
4. Logs and error messages
5. Reproducibility steps

## Post-Deployment Tasks

### Week 1
- Monitor error logs
- Test with varied brain content
- Gather performance metrics
- Collect user feedback

### Week 2-4
- Analyze entity extraction patterns
- Optimize pattern definitions
- Implement caching strategy
- Plan ML improvements

### Month 2+
- Implement semantic similarity
- Add graph visualization
- Build advanced query language
- Integrate with more systems

## Success Criteria

✅ All endpoints responding successfully
✅ Parsing accuracy > 80% (review samples)
✅ Graph builds in < 2 seconds for 1000 nodes
✅ API response times < 100ms
✅ Zero critical errors in logs
✅ Memory usage stable
✅ User feedback positive

---

**Deployment Status**: Ready for Production
**Risk Level**: Low (backward compatible)
**Rollback Time**: < 5 minutes
**Expected Downtime**: < 1 minute
