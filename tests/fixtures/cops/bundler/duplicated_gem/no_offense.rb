source 'https://rubygems.org'
gem 'rubocop'
gem 'flog'
gem 'rails'
gem 'puma'
gem 'nokogiri'

if ENV['RAILS_VERSION'] == '5.2'
  gem 'sqlite3', '< 1.5', require: false
elsif ENV['RAILS_VERSION'] == '6.0'
  gem 'sqlite3', '1.5.1'
else
  gem 'sqlite3', '< 2', require: false
end

case
when ENV['RUBOCOP_VERSION'] == 'master'
  gem 'reek', git: 'https://github.com/troessner/reek.git'
else
  gem 'reek', '~> 6.0'
end

# Modifier if inside case/when should not create a separate conditional root
case rails_version.segments[0]
when 6
  gem 'concurrent-ruby', '< 1.3.5'
when 7
  gem 'concurrent-ruby', '< 1.3.5' if rails_version.segments[1] < 2
end

# Gems in if/else where the else branch wraps in a single-statement group block
# RuboCop treats the gem as a direct child of the else branch (Parser gem quirk:
# single-statement block bodies expose their child as a direct child_node of the block)
if ENV["ALLOW_DEV_POPULATE"] == "1"
  gem "faker"
else
  group :development, :test do
    gem "faker"
  end
end

# Duplicated gem inside if...end (no else) — conditional exemption
if RUBY_VERSION >= "3.2.0"
  gem "minitest-mock"
  gem "async", "~>2.0"
  gem "minitest-mock"
end

# Structural equality exemption: when the FIRST declaration is in a conditional
# and other declarations have identical source, RuboCop's `within_conditional?`
# uses structural `==` to match them. The conditional must come first because
# `conditional_declaration?` checks `nodes[0]`'s ancestor.
if ENV["CI"]
  gem "redcarpet"
end
group :development do
  gem "redcarpet"
end

# Same pattern with case/when (conditional first)
case RUBY_ENGINE
when "ruby"
  gem "simplecov"
when "jruby"
  gem "simplecov"
end
group :test do
  gem "simplecov"
end

# Nested if inside else of an if/elsif chain — all gems are within
# branches of the root conditional via child_nodes structural equality
if ENV['RAILS'] >= "8.0"
  gem 'mysql2', '~> 2.1'
elsif ENV['RAILS'] >= "7.1"
  gem 'mysql2', '~> 1.7'
else
  if ENV['RAILS'] >= "6.0"
    gem 'mysql2', '~> 1.4'
  else
    gem 'mysql2', '~> 1.3'
  end
end

# Case/when/else with nested if/else — gems in nested conditional branches
# are found via structural equality (child_nodes.include? in Parser gem).
# Each gem matches a structurally-equal child in some case branch.
case rails_version
when "master"
  rails = { github: "rails/rails" }
  gem 'sass-rails', '>= 4.0.2'
when "default"
  rails = ">= 3.1.0"
  gem 'sass-rails'
else
  rails = "~> #{rails_version}"
  if rails_version[0] == '4'
    gem 'sass-rails', '>= 4.0.2'
  else
    gem 'sass-rails'
  end
end
