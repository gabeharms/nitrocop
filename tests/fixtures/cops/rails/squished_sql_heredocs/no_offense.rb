<<-SQL.squish
  SELECT * FROM posts
    WHERE id = 1
SQL

<<~SQL.squish
  SELECT * FROM posts
    WHERE id = 1
SQL

<<-SQL
  SELECT * FROM posts
  -- This is a comment, so squish can't be used
    WHERE id = 1
SQL

execute(<<~SQL.squish, "Post Load")
  SELECT * FROM posts
    WHERE post_id = 1
SQL

<<~SQL
  SELECT * FROM posts
  WHERE id = 1 -- filter active records
SQL

<<-SQL
  SELECT name FROM users
  WHERE role = 'admin' -- only admins
  ORDER BY name
SQL

<<~SQL
  SELECT "col--name" FROM posts
  WHERE status = 1 -- inline comment
SQL

# Quoted tags with squish are fine
<<~'SQL'.squish
  SELECT * FROM records
SQL

<<-'SQL'.squish
  SELECT id FROM items
SQL

# .squish chained on the line after the closing heredoc tag
expected_query = <<-SQL
  SELECT recommendations.* FROM recommendations
  LEFT OUTER JOIN people ON people.id = recommendations.person_id
  WHERE (people.name = 'Ernie' AND parents_people.name = 'Test')
SQL
.squish

query = <<-SQL
  (SELECT MAX(articles.title)
     FROM articles
    WHERE articles.person_id = people.id
    GROUP BY articles.person_id
  )
SQL
.squish

join = <<-SQL
  LEFT OUTER JOIN (
      SELECT comments.*, row_number() OVER (PARTITION BY comments.article_id ORDER BY comments.id DESC) AS rownum
      FROM comments
    ) AS latest_comment
    ON latest_comment.article_id = articles.id
SQL
.squish

# Regex-based quote stripping creates phantom -- from adjacent dashes (matches RuboCop behavior)
DB.exec(<<~SQL)
    UPDATE groups SET flair_icon = REPLACE(REPLACE(flair_url, 'fas fa-', ''), ' fa-', '-')
    WHERE flair_url LIKE 'fa%'
  SQL
