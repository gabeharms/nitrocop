puts Regexp.last_match(1)

Regexp.last_match(9)

Regexp.last_match(0)

module Namespace
  name = ::Regexp.last_match.post_match

  id = ::Regexp.last_match.pre_match

  hashed_password_without_type = ::Regexp.last_match.post_match

  ::Regexp.last_match(0)

  content = ::Regexp.last_match.post_match

  raw_record = ::Regexp.last_match.post_match

  raw_record = ::Regexp.last_match.post_match

  raw_action = ::Regexp.last_match.post_match
end
