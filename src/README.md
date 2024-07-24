# Architecture

- No calls to database in the matching engine hot loop.
- Requires duplication of state, need to keep at least two copies of order books.

## Services

- Services that run in the background and ingest queues

## API

- models to json
- submit order
- cancel order
- modify order


## Models

- Interacts with database

## Web

- Everything relate to the front end
    - render to html
    - session management
