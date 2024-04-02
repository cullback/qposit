# API

- Routes for api requests.
- Parsing from payloads to internal types.

We redefine types since
- they may be a subset of what the underlying structure provides
- underlying types don't need to derive serde / openapi traits