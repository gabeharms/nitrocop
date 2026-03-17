begin
  foo
  rescue => e
  ^^^^^^ Layout/RescueEnsureAlignment: Align `rescue` with `begin`.
  bar
end

begin
  foo
  ensure
  ^^^^^^ Layout/RescueEnsureAlignment: Align `ensure` with `begin`.
  bar
end

begin
  baz
  rescue => e
  ^^^^^^ Layout/RescueEnsureAlignment: Align `rescue` with `begin`.
  qux
  ensure
  ^^^^^^ Layout/RescueEnsureAlignment: Align `ensure` with `begin`.
  grault
end

# Misaligned rescue inside def
def fetch(key)
  @store[key]
    rescue
    ^^^^^^ Layout/RescueEnsureAlignment: Align `rescue` with `def`.
    raise RetrievalError
end

# Misaligned ensure inside def
def process
  do_work
    ensure
    ^^^^^^ Layout/RescueEnsureAlignment: Align `ensure` with `def`.
    cleanup
end

# Misaligned rescue + ensure inside def
def transform
  compute
    rescue StandardError => e
    ^^^^^^ Layout/RescueEnsureAlignment: Align `rescue` with `def`.
    handle(e)
    ensure
    ^^^^^^ Layout/RescueEnsureAlignment: Align `ensure` with `def`.
    finalize
end

# Misaligned rescue inside class
class Widget
  do_something
    rescue
    ^^^^^^ Layout/RescueEnsureAlignment: Align `rescue` with `class`.
    handle_error
end

# Misaligned ensure inside module
module Helpers
  initialize_stuff
    ensure
    ^^^^^^ Layout/RescueEnsureAlignment: Align `ensure` with `module`.
    finalize
end

# Misaligned rescue inside do-end block
items.each do |item|
  item.process
    rescue StandardError
    ^^^^^^ Layout/RescueEnsureAlignment: Align `rescue` with `do`.
    next
end

# Misaligned ensure inside do-end block
records.map do |rec|
  rec.save
    ensure
    ^^^^^^ Layout/RescueEnsureAlignment: Align `ensure` with `do`.
    rec.unlock
end

# Rescue chain: second rescue misaligned in begin
begin
  risky_call
rescue Timeout::Error
  retry
  rescue
  ^^^^^^ Layout/RescueEnsureAlignment: Align `rescue` with `begin`.
end
