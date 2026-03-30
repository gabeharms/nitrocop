# frozen_string_literal: true
hash.fetch(:key, 5)

hash.fetch(:key, :value)

hash.fetch(:key, 'default')

# String default with frozen_string_literal: true is flagged
val = hash.fetch(:key, 'fallback')

# Unary minus with space is still a simple literal
@file_limit = options.fetch(:file_limit, - 1)
