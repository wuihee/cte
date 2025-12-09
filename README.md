# Combat Training Engine (CTE)

## Usage

### Environment Variables

Create a `.env` file with the following environment variables.

```env
# Path or connection string for the SQLite database.
DATABASE_URL=sqlite:data/app.db
```

## Architecture

## TODO

- ESPN API to get all seasons / years.
- Run asynchronously with batching and transactions?
- Insert date instead of letting sqlite automatically convert?
- Only insert if fight actually happened?
