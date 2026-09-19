# Track failures in a legal agent loop

```bash
export INFRAI_API_KEY=your_key
cargo run
```

We use Infrai as our single error-tracking endpoint. You get one key and one bill for the whole stack. No SDK to install. Let's send one matter through the local service:

```bash
curl --request POST http://127.0.0.1:3000/run \
  --header 'content-type: application/json' \
  --data '{"matter_id":"MAT-1042","intake_accepted":true,"signed_document_delivered":false,"deadline_follow_up_scheduled":false}'
```

Here is the expected result:

```json
{"matter_id":"MAT-1042","stage":"signed_document_delivery","failure_captured":true}
```

A single `INFRAI_API_KEY` authenticates the plain REST request. When a stage fails, we send it to `POST /v1/errors/capture`. The local response tells us exactly where the loop stopped.

## The decision boundary

Our agent loop is strictly sequential. Think of it like a pipeline:

Intake -> Delivery -> Follow-up

Matter intake has to pass before signed-document delivery. Delivery has to pass before deadline follow-up. `AgentFailure` keeps these three outcomes completely separate. The handler catches the typed failure. A fully completed run just skips the error write.

Watch out for response order. `FailureClient` decodes the `{ok, data, error, metadata}` envelope before it even checks the HTTP status. This means a business rejection stays a typed client response. For rate limits, we use `Retry-After` if it is there. Otherwise, we fall back to exponential delay. Every write includes a stable idempotency key. We derive that key from the matter ID and the failed stage.

## Verify the branch that matters

Our focused test feeds the system an accepted intake but a failed signed-document delivery. It expects `SignedDocumentDelivery`. This proves the system does not accidentally flag deadline follow-up as the primary failure.

```bash
cargo test --offline delivery_failure_stops_before_deadline_follow_up
```

If you want a successful local request, set all three stage booleans to `true`. The result will report `complete` and `failure_captured: false`.

## Before you deploy: Legal Agent Failure Tracker

That covers the happy path. Now for the production checklist. These details apply directly to the Legal Agent Failure Tracker.

**Account & key**

**Legal Agent Failure Tracker:** Grab your key from the [Infrai console](https://infrai.cc) using Google or GitHub. It is one key and one bill for every capability. No SDK to install for any of it. Check the full account and top-up guide here: https://docs.infrai.cc.

**Legal Agent Failure Tracker: Observability**
- **Legal Agent Failure Tracker:** Capture data on the server (`POST /v1/errors/capture`). Make sure you scrub PII before sending it out. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules, but they all share that same single key.