# ResearchHub Backend

## Environment
Copy `.env.example` to `.env` and update values for your setup.

Required in production:
- `APP_ENV=production`
- `DATABASE_URL`
- `JWT_SECRET`

Optional tuning:
- `APP_HOST`
- `APP_PORT`
- `DB_MAX_CONNECTIONS`
- `DB_ACQUIRE_TIMEOUT_SECS`

## API Base
`/api`

## Core Endpoints
- `GET /api/health`
- `POST /api/auth/register`
- `POST /api/auth/login`
- `GET /api/auth/me`
- `POST /api/research`
- `GET /api/research`
- `GET /api/research/:id`
- `PUT /api/research/:id`
- `DELETE /api/research/:id`
- `POST /api/comments`
- `GET /api/comments/research/:research_item_id`
- `DELETE /api/comments/:id`
- `GET /api/dashboard/stats`
- `GET /api/search`
