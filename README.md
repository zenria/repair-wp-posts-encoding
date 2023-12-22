# repair-wp-posts-encoding

Pile of encoding hacks to repair a Wordpress `wp_posts` table broken due
to mixed encoding (UTF-8/latin1) in a latin1 table.

The table has been broken after an update of mysql. Wordpress used to store utf8 text but after db update all the tables
are said to be latin1 encoded thus breaking utf8 text stored in them.

The trick is:

- convert latin1 columns to binary
- check if the columns contains valid utf-8, if not repair broken value
- convert binary columns to utf8

To run this tool, you need to set an environment variable: `DATABASE_URL`.
