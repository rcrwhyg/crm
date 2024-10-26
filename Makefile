export:
	@pg_dump -U alon --table=export_user_stats --data-only --column-inserts stats > user-stat/fixtures/data.sql
