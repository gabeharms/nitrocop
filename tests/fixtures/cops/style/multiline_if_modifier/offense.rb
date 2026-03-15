{
^ Style/MultilineIfModifier: Favor a normal if-statement over a modifier clause in a multiline statement.
  result: run
} if cond

{
^ Style/MultilineIfModifier: Favor a normal unless-statement over a modifier clause in a multiline statement.
  result: run
} unless cond

[
^ Style/MultilineIfModifier: Favor a normal if-statement over a modifier clause in a multiline statement.
  1,
  2,
  3
] if condition

raise "Value checking error" \
^ Style/MultilineIfModifier: Favor a normal if-statement over a modifier clause in a multiline statement.
    ' for attribute' if value.nil?

raise "Type checking error" \
^ Style/MultilineIfModifier: Favor a normal unless-statement over a modifier clause in a multiline statement.
  "for attribute" unless acceptable

fail "Association defined" \
^ Style/MultilineIfModifier: Favor a normal if-statement over a modifier clause in a multiline statement.
     "a second time" if duplicate?(name)

log_error("Returned nil body. " \
^ Style/MultilineIfModifier: Favor a normal if-statement over a modifier clause in a multiline statement.
          "Probably wanted empty string?") if @response.body.nil?

help! "You do not have permission" \
^ Style/MultilineIfModifier: Favor a normal unless-statement over a modifier clause in a multiline statement.
      'Perhaps try sudo.' unless File.writable?(dir)

# Explicit + operator with backslash continuation
raise "Must specify backup file to " + \
^ Style/MultilineIfModifier: Favor a normal if-statement over a modifier clause in a multiline statement.
  "convert to the new model" if filename.nil?

# Assignment with backslash continuation
@batch_size ||= \
^ Style/MultilineIfModifier: Favor a normal if-statement over a modifier clause in a multiline statement.
  ENV['BATCH_SIZE'].to_i if environment['BATCH_SIZE']

# Multiple backslash continuations (3+ lines)
raise StandardError, "spoofing attack?! " \
^ Style/MultilineIfModifier: Favor a normal if-statement over a modifier clause in a multiline statement.
  "CLIENT=#{@req.client_ip} " \
  "FORWARDED=#{@req.x_forwarded_for}" if @req.forwarded_for.any?
