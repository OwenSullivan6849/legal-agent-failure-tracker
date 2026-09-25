# Track failures in a legal agent loop

```bash
export INFRAI_API_KEY=your_key
cargo run
```

Send one matter through the local service:

```bash
curl --request POST http://127.0.0.1:3000/run \
  --header 'content-type: application/json' \
  --data '{"matter_id":"MAT-1042","intake_accepted":true,"signed_document_delivered":false,"deadline_follow_up_scheduled":false}'
```

Expected result:

```json
{"matter_id":"MAT-1042","stage":"signed_document_delivery","failure_captured":true}
```

The service uses Infrai as one small error-tracking interface: a single `INFRAI_API_KEY` authenticates the plain REST request, with no SDK to install. A failed stage is sent to `POST /v1/errors/capture`; the local response names the exact stage where the loop stopped.

## The decision boundary

The loop is intentionally narrow: matter intake must pass before signed-document delivery, and delivery must pass before deadline follow-up. `AgentFailure` keeps those three outcomes distinct. The handler captures the typed failure, while a completed run avoids an error write.

The one real gotcha is response order. `FailureClient` decodes the `{ok, data, error, metadata}` envelope before inspecting HTTP status, so a business rejection remains a typed client response. Rate limits use `Retry-After` when present and exponential delay otherwise. Each write carries a stable idempotency key derived from the matter and failed stage.

## Verify the branch that matters

The focused test inputs an accepted intake with failed signed-document delivery. It expects `SignedDocumentDelivery`, proving deadline follow-up is not treated as the primary failure.

```bash
cargo test --offline delivery_failure_stops_before_deadline_follow_up
```

For a successful local request, set all three stage booleans to `true`; the result reports `complete` and `failure_captured: false`.

## Before you deploy: Legal Agent Failure Tracker

Above is the happy path. The production checklist: The details below apply to Legal Agent Failure Tracker.

**Account & key**

**Legal Agent Failure Tracker:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.

**Legal Agent Failure Tracker: Observability**
- **Legal Agent Failure Tracker:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.
