def foo
  @bar ||= compute
  ^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@bar` does not match method name `foo`.
end
def something
  @other ||= calculate
  ^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@other` does not match method name `something`.
end
def value
  @cached ||= fetch
  ^^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@cached` does not match method name `value`.
end
def issue_token!
  return @token if defined?(@token)
                            ^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@token` does not match method name `issue_token!`. Use `@issue_token` instead.
         ^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@token` does not match method name `issue_token!`. Use `@issue_token` instead.
  @token = create_token
  ^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@token` does not match method name `issue_token!`. Use `@issue_token` instead.
end
define_method(:values) do
  @foo ||= do_something
  ^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@foo` does not match method name `values`.
end
klass.define_method(:values) do
  @bar ||= do_something
  ^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@bar` does not match method name `values`.
end
define_singleton_method(:values) do
  @baz ||= do_something
  ^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@baz` does not match method name `values`.
end
def self.records
  @other ||= fetch_records
  ^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@other` does not match method name `records`.
end
def url_helpers
  if supports_path
    generate_url_helpers(true)
  else
    @helpers_without_paths ||= generate_url_helpers(false)
    ^^^^^^^^^^^^^^^^^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@helpers_without_paths` does not match method name `url_helpers`.
  end
end
def list_users
  @username_cache ||= @user_provider.list_users unless @user_provider.nil?
  ^^^^^^^^^^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@username_cache` does not match method name `list_users`.
end
def size
  case @value
  when String
    @value.bytesize
  else
    @cached_size ||= Marshal.dump(@value).bytesize
    ^^^^^^^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@cached_size` does not match method name `size`.
  end
end
def ensure_subscribed
  @mutex.synchronize do
    @subscriber ||= ActiveSupport::Notifications.subscribe(self)
    ^^^^^^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@subscriber` does not match method name `ensure_subscribed`.
  end
end
def infer_decoder
  compute_decoder
rescue => e
  warn(e)
ensure
  @decoder ||= :itself.to_proc
  ^^^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@decoder` does not match method name `infer_decoder`.
end
def param_encode_language(value = nil)
  value ? @encode_language = value : @encode_language ||= 'en'
                                     ^^^^^^^^^^^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@encode_language` does not match method name `param_encode_language`.
end
def set_items exp = nil
  if exp
    @items ||= []
    ^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@items` does not match method name `set_items`.
    @items.concat [exp]
  else
    @items ||= []
    ^^^^^^ Naming/MemoizedInstanceVariableName: Memoized variable `@items` does not match method name `set_items`.
  end
end
